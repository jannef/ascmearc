//! Objects: the objects, their item classes, shops, equipment slots, materials, and the pack.

use super::order;
use super::util::{Term, anchored, article, at_normal_pace, capital, glyph, yes};
use ascmearc_core::content::{BASE_TURN, Content, ItemClassId, ObjectDef, ObjectId, price};
use ascmearc_core::object::format_weight;
use ascmearc_core::text::and_list;
use std::fmt::Write as _;

/// The equipment slots in the order the screen lists them, with their letters, what goes in
/// each, and how long putting a thing on there or taking it off takes. A class's own slot (the
/// aristocrat's extra rings) says whose it is, with its letter for them; every other slot has
/// the letter it has for everyone else.
pub(super) fn equipment(content: &Content) -> String {
    let mut out = String::from(
        "| Letter | Slot | What goes there | On or off takes |\n|:---:|---|---|---:|\n",
    );
    for (slot, def) in content.slot_ids() {
        // Its letter for whoever has it; for an ordinary slot also the letter of each class
        // whose own slots come before it ("`j`, an aristocrat's `m`").
        let letter_for = |class: Option<&str>| {
            let i = (content.slots_for(class))
                .position(|(s, _)| s == slot)
                .unwrap_or_default();
            char::from(b'a' + i as u8)
        };
        let mut letter = format!("`{}`", letter_for(def.class.as_deref()));
        if def.class.is_none() {
            let mut classes: Vec<&str> = (content.slots().iter())
                .filter_map(|s| s.class.as_deref())
                .collect();
            classes.dedup();
            for class in classes {
                let theirs = letter_for(Some(class));
                if theirs != letter_for(None) {
                    let name = content.class(class).map_or(class, |c| c.name.as_str());
                    let _ = write!(
                        letter,
                        ", {} {}'s `{theirs}`",
                        article(name),
                        name.to_lowercase()
                    );
                }
            }
        }
        let name = match &def.class {
            Some(class) => {
                let plural = content
                    .class(class)
                    .map_or(class.clone(), |c| format!("{}s", c.name.to_lowercase()));
                let page = super::character::class_page(class);
                format!("{} ([{plural}]({page}) only)", def.name)
            }
            None => def.name.clone(),
        };
        let fits: Vec<String> = (order::objects(content).into_iter())
            .filter(|&(id, _)| content.fits_properly(id, slot))
            .map(|(_, o)| {
                let mut name = o.name.clone();
                if o.two_handed {
                    name.push_str(" (both hands)");
                }
                if let Some(kind) = &o.launches {
                    let ammo =
                        (content.ammunition(kind)).map_or(kind.clone(), |a| a.plural.clone());
                    let _ = write!(name, " (shoots {ammo})");
                }
                name
            })
            .collect();
        let mut fits = if fits.is_empty() {
            "-".to_string()
        } else {
            capital(&fits.join(", "))
        };
        if def.takes_weapons {
            fits.push_str(", any other weapon, and anything made for throwing");
        }
        let cost = content.action_costs().cost(def.action);
        let _ = writeln!(
            out,
            "| {letter} | {name} | {fits} | {} |",
            at_normal_pace(price(cost, BASE_TURN))
        );
    }
    out
}

/// Where an object goes, if it can be worn or wielded: the slots that fit it.
pub(super) fn goes_on(content: &Content, id: ObjectId) -> String {
    // Where it belongs; else where it may be readied (rocks: the missile weapon slot).
    let names = |proper: bool| -> Vec<String> {
        // Everyone's slots: the extra rings of an aristocrat's are in the slot table.
        (content.slots_for(None))
            .filter(|&(s, _)| {
                if proper {
                    content.fits_properly(id, s)
                } else {
                    content.fits(id, s)
                }
            })
            .map(|(_, s)| s.name.to_lowercase())
            .collect()
    };
    let slots = Some(names(true))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| names(false));
    if slots.is_empty() {
        return "-".into();
    }
    let both = if content.object(id).two_handed {
        " (both hands)"
    } else {
        ""
    };
    format!("{}{both}", capital(&slots.join(" or ")))
}

/// The item classes in the order item lists group by, each with the objects of it (by name, in
/// the plural; corpses as one entry).
pub(super) fn item_classes(content: &Content) -> String {
    let mut out = String::from("| Class | Objects of it |\n|---|---|\n");
    for (i, class) in content.item_classes().iter().enumerate() {
        let of_it = |o: &&ObjectDef| usize::from(o.class.0) == i;
        let objects = content.object_defs().iter().filter(of_it);
        let mut named: Vec<&ObjectDef> = objects.clone().filter(|o| o.corpse.is_none()).collect();
        named.sort_by_cached_key(|o| o.name.to_lowercase());
        let mut names: Vec<String> = named.iter().map(|o| o.plural.clone()).collect();
        if objects.clone().any(|o| o.corpse.is_some()) {
            names.push("corpses".into());
        }
        let names = if names.is_empty() {
            "-".to_string()
        } else {
            capital(&names.join(", "))
        };
        let _ = writeln!(
            out,
            "| {} | {names} |",
            anchored(Term::ItemClass, &class.id, &class.name)
        );
    }
    out
}

/// Every object but corpses (Food and corpses has those), by class, then name.
pub(super) fn objects(content: &Content) -> String {
    let mut out = String::from(
        "| Object | Glyph | Class | Made of | Weight (each) | Worth (each) | Stacks | Worn or wielded on |\n\
         |---|:---:|---|---|---:|---:|:---:|---|\n",
    );
    let loose = content.gems().map(|g| g.object);
    for (id, o) in order::objects(content) {
        if o.corpse.is_some() {
            continue;
        }
        let worth = match o.value {
            0 => "nothing".to_string(),
            v => format!("{v} gold"),
        };
        // A loose gem is made of its gem, which weighs and prices it (Gems has the tables).
        let (made, weight, worth) = match (loose == Some(id), content.gems()) {
            (true, Some(gems)) => {
                let grams = |s: Option<&ascmearc_core::content::GemSize>| s.map_or(0, |s| s.weight);
                let (least, most) = (grams(gems.sizes().first()), grams(gems.sizes().last()));
                (
                    "its gem".to_string(),
                    format!("{least}–{most} g"),
                    "its gem's".to_string(),
                )
            }
            _ => (
                content.describe_parts(&o.parts),
                format_weight(u64::from(o.weight)),
                worth,
            ),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {made} | {weight} | {worth} | {} | {} |",
            capital(&o.name),
            glyph(o.glyph),
            content.item_class(o.class).name.to_lowercase(),
            yes(o.stackable),
            goes_on(content, id)
        );
    }
    out
}

/// The kinds of shop: what each sells and buys, by class, and its prices in
/// words of the value (an object's worth, in the objects table).
pub(super) fn shops(content: &Content) -> String {
    let mut out =
        String::from("| Shop | Sign | Sells | Buys | Asks | Pays |\n|---|---|---|---|---:|---:|\n");
    let Some(shops) = content.shops() else {
        return out;
    };
    let classes = |list: &[ItemClassId]| {
        let names: Vec<String> = (list.iter())
            .map(|&c| content.item_class(c).name.to_lowercase())
            .collect();
        if names.is_empty() {
            return "nothing".to_string();
        }
        and_list(&names)
    };
    for def in shops.defs() {
        // What a generated one always has, first: "scrolls of identify always".
        let always: Vec<String> = (def.always.iter())
            .map(|&o| content.object(o).plural.clone())
            .collect();
        let stock = match always.as_slice() {
            [] => classes(&def.stock),
            _ => format!("{} ({} always)", classes(&def.stock), always.join(", ")),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {}% of the worth | {}% of the worth |",
            capital(&def.name),
            def.sign("Grimbold"),
            stock,
            classes(&def.buys),
            def.markup,
            def.pays
        );
    }
    out
}

/// How shops restock as the player gains levels (`data/rules/restock.toml`, #1735): what goes and
/// comes, the item level of the new goods by the player's level, and each town's bonuses
/// (`data/towns/`, `[town.restock]`).
pub(super) fn restock(content: &Content) -> String {
    let Some(rules) = content.restock() else {
        return "Shops never restock.\n".to_string();
    };
    let hundredths = |n: u32| {
        let whole = format!("{}.{:02}", n / 100, n % 100);
        whole
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };
    let range = |(lo, hi): (u8, u8)| match lo == hi {
        true => lo.to_string(),
        false => format!("{lo} to {hi}"),
    };
    let mut out = String::from("| Each level you gain | How much |\n|---|---|\n");
    let _ = writeln!(out, "| Stacks of goods that go | {} |", range(rules.gone));
    let _ = writeln!(
        out,
        "| New stacks that come | {}, plus anything the shop always has that is gone |",
        range(rules.come)
    );
    let _ = writeln!(
        out,
        "| Item level of the new goods | 1, plus {} for each of your levels beyond the first |",
        hundredths(rules.per_level)
    );
    let _ = writeln!(
        out,
        "| A shop outside a town | plus {}% of its level's item level |",
        rules.danger
    );
    let scale = rules.invest;
    let _ = writeln!(
        out,
        "| A merchant's stake | plus {} for each {} gold it is worth, at most {} |",
        hundredths(scale.step),
        scale.per,
        hundredths(scale.most)
    );
    out.push_str("\nThe item level of a town shop's new goods with no bonus, rounded down:\n\n");
    let levels = [1u8, 5, 10, 20, 30, 40, 50];
    out.push_str("| Your level |");
    for level in levels {
        let _ = write!(out, " {level} |");
    }
    out.push_str("\n|---|");
    out.push_str(&"---:|".repeat(levels.len()));
    out.push_str("\n| Item level |");
    for level in levels {
        let _ = write!(out, " {} |", rules.item_level(level, 0, 0));
    }
    out.push_str("\n\nWhat raises a town's restocking, in item levels:\n\n");
    out.push_str("| Town | What | Item levels |\n|---|---|---|\n");
    for town in content.towns() {
        let Some(restock) = &town.restock else {
            continue;
        };
        for (quest, &bonus) in &restock.quests {
            let name = (content.quest_id(quest)).map_or(quest.as_str(), |q| &content.quest(q).name);
            let _ = writeln!(
                out,
                "| {} | {name} done | plus {} |",
                town.name,
                hundredths(bonus)
            );
        }
        if let Some(invest) = &restock.invest {
            let after = (invest.after.as_deref())
                .and_then(|q| content.quest_id(q))
                .map(|q| format!(", once {} is done", content.quest(q).name))
                .unwrap_or_default();
            let share = match invest.share {
                100 => "all of it".to_string(),
                50 => "half of it".to_string(),
                n => format!("{n}% of it"),
            };
            let _ = writeln!(
                out,
                "| {} | A merchant's stake | {share}{after} |",
                town.name
            );
        }
    }
    out
}

/// What magic adds to an item's worth (`data/rules/magic_worth.toml`): by rarity, then by each
/// identified affix's tier level (the affixes' tiers run to item level 12).
pub(super) fn magic_worth(content: &Content) -> String {
    let mut out = String::from("| Rarity | Worth added |\n|---|---:|\n");
    let Some(rules) = content.magic_worth() else {
        return out;
    };
    let _ = writeln!(out, "| Magic | {}% |", rules.rarity.magic);
    let _ = writeln!(out, "| Rare | {}% |", rules.rarity.rare);
    out.push_str("\n| Tier's item level |");
    let levels = 1..=12u8;
    for level in levels.clone() {
        let _ = write!(out, " {level} |");
    }
    out.push_str("\n|---|");
    out.push_str(&"---:|".repeat(levels.clone().count()));
    out.push_str("\n| Gold per affix |");
    for level in levels {
        let _ = write!(out, " {} |", rules.affix_worth(level));
    }
    out.push('\n');
    out
}

/// The player's own weight (the pack holds any number of stacks: weight is its only limit).
pub(super) fn pack(content: &Content) -> String {
    let player = content
        .creature_id("player")
        .map_or(0, |id| content.creature(id).weight);
    format!(
        "| Fact | Value |\n|---|---|\n| Your own weight | {} |\n",
        format_weight(u64::from(player))
    )
}
