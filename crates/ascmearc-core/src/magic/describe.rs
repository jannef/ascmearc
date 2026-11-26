//! What an item is, in words, as the player knows it: its base stats, and its affixes once it is
//! identified. The item description screen shows these lines; they read an item only through
//! [`Stack::known_magic`], so an unidentified item's affixes cannot leak.
//!
//! Numbers that are item properties are shown, as in Path of Exile (dice, criticals, Defence
//! bonuses, Protection, an affix's rolled value); speed is said in words ("attacks faster") and
//! penetration only as a fact ("Pierces armour"), never as a number, and nothing about who wears
//! it (no to-hit total, no modifier).

use super::{AffixKind, Effect, Rolled};
use crate::combat::HIGHEST;
use crate::content::{
    ActionKind, Content, Draught, ItemKind, ObjectDef, Rarity, data_line, weapon_noun_phrase,
};
use crate::lang::{Arg, MessageId, Msg, Phrase};
use crate::msg;
use crate::object::{Stack, format_weight};
use crate::scroll_label::Lore;
use crate::world::{Actor, CastBy};

/// What a line of a description is, for how it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    /// What it is: rarity, base, what it is made of, weight.
    Base,
    /// A base stat: damage, Defence, range.
    Stat,
    /// That its magic is unidentified (the item level is never shown: a hidden number).
    Note,
    /// An affix: its kind ("Prefix"), name, and what it does.
    Affix(AffixKind),
    /// A unique item's power, in words ("Its every cut poisons.").
    Power,
    /// A unique item's lore: the last line.
    Lore,
}

/// One line of a description: a message (`messages/describe.toml`), which the screen renders
/// in the player's language and the wiki and the tests in English ([`Line::english`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub kind: LineKind,
    pub msg: Msg,
}

impl Line {
    /// The line in English, keys left written: the tests' and the dumps'.
    pub fn english(&self, content: &Content) -> String {
        crate::messages::english_msg(content, &self.msg)
    }
}

fn line(kind: LineKind, msg: impl Into<Msg>) -> Line {
    Line {
        kind,
        msg: msg.into(),
    }
}

/// A count or a value as a message's number.
fn num(n: impl TryInto<i64>) -> Arg {
    Arg::Number(n.try_into().unwrap_or(i64::MAX))
}

/// `items` said one after another with `join` (`listed`, `and_joined`, `or_joined`): the
/// language owns the separator. `None` for none.
/// `parts` in words, as [`Content::describe_parts`] writes them: every part, the main one first,
/// each its material and part ("steel blade", `describe.part_made`), an unnamed part its material
/// ("steel"), joined with commas (`describe.comma_joined`).
fn parts_arg(content: &Content, parts: &[crate::content::Part]) -> Arg {
    let words: Vec<Arg> = (parts.iter())
        .map(|p| {
            let material = Phrase::id(format!("material:{}", content.material(p.material).id));
            match p.part {
                Some(part) => {
                    let part = &content.part_defs()[usize::from(part.0)].id;
                    (Msg::new(msg::describe::PART_MADE).role("material", material))
                        .role("part", Phrase::id(format!("part:{part}")))
                        .into()
                }
                None => material.into(),
            }
        })
        .collect();
    joined(words, msg::describe::COMMA_JOINED).unwrap_or_else(Arg::empty)
}

fn joined(items: Vec<Arg>, join: MessageId) -> Option<Arg> {
    let mut items = items;
    let last = items.pop()?;
    Some(items.into_iter().rev().fold(last, |rest, first| {
        Msg::new(join.clone())
            .role("first", first)
            .role("rest", rest)
            .into()
    }))
}

/// What a description shows of worth (Gem lore): nothing unless the player
/// has an eye for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Appraisal {
    /// What each gem in it is worth.
    pub gems: bool,
    /// How many enchantments it can bear (docs/dev/enchanting.md): for a character trained in
    /// Appraise.
    pub capacity: bool,
}

/// [`lines`] for a player with an eye for worth: each gem's worth after its line (`Pommel: small
/// emerald, fine cut (worth 40 gold)`), as `appraisal` shows.
/// An unidentified scroll is called by its label, as `lore` names it.
pub fn appraised(stack: &Stack, lore: Lore<'_>, appraisal: Appraisal) -> Vec<Line> {
    appraised_for(stack, lore, appraisal, None, None)
}

/// [`appraised`] as `user` would use it: a wand whose type is known says what it does in their
/// hands ([`wand_does`]); a maker's mark is `maker`'s, the player's name ([`mark_line`]).
pub(crate) fn appraised_for(
    stack: &Stack,
    lore: Lore<'_>,
    appraisal: Appraisal,
    user: Option<&Actor>,
    maker: Option<&str>,
) -> Vec<Line> {
    let content = lore.content;
    let mut out = lines_named(stack, content, lore.names_phrase(stack), maker);
    if appraisal.capacity
        && let Some(said) = capacity_line(stack, content)
    {
        // After the affixes, before a unique's power and lore.
        let at = (out.iter())
            .rposition(|l| !matches!(l.kind, LineKind::Power | LineKind::Lore))
            .map_or(out.len(), |i| i + 1);
        out.insert(at, said);
    }
    out.extend(grimoire_lines(stack, lore, user));
    out.extend(wand_lines(stack, lore, user));
    out.extend(rod_lines(stack, lore));
    out.extend(potion_lines(stack, lore));
    // The first line says what it is; a line for each gem in it follows.
    let gems_at = 1;
    let parts = stack.parts(content);
    if appraisal.gems
        && let Some(gems) = content.gems()
    {
        let worths = (parts.iter()).filter_map(|p| {
            let gem = content.material(p.material).gem.as_ref();
            p.gem
                .map(|cutting| gem.map_or(0, |g| gems.value(g, cutting)))
        });
        for (line, worth) in out[gems_at..].iter_mut().zip(worths) {
            line.msg = (Msg::new(msg::describe::GEM_WORTH).role("line", line.msg.clone()))
                .role("worth", num(worth));
        }
    }
    out
}

/// How many enchantments `stack` can bear, in words (docs/dev/enchanting.md), for a base item
/// (one with a `kind`): what it can bear while it is plain or its magic unidentified (`It can
/// bear two enchantments.`), else what it bears against that (`It bears one enchantment and
/// could bear one more.`, `It bears all it can.`, `It bears more than its make should hold.`).
fn capacity_line(stack: &Stack, content: &Content) -> Option<Line> {
    content.object(stack.object).kind?;
    let room = usize::from(super::capacity(stack, content));
    let said = match stack.known_magic().map(|m| m.affixes.len()) {
        None => Msg::new(msg::describe::CAPACITY_CAN).role("room", enchantments(room)),
        // A unique holds what its table sets, past its make's capacity too (#2115).
        Some(n) if n > room && !stack.is_unique() => msg::describe::CAPACITY_OVER.into(),
        Some(n) if n > room => msg::describe::CAPACITY_ALL.into(),
        Some(n) if n == room => msg::describe::CAPACITY_ALL.into(),
        Some(n) => (Msg::new(msg::describe::CAPACITY_MORE).role("has", enchantments(n)))
            .role("n", num(room - n)),
    };
    Some(line(LineKind::Stat, said))
}

/// Whose mark `stack` bears (Maker's mark, the craftsperson's): "It bears Brakka's mark.", the
/// maker being the player (no one else makes marked work); "It bears its maker's mark." where
/// no name is known. `None` for an unmarked item.
fn mark_line(stack: &Stack, maker: Option<&str>) -> Option<Line> {
    stack.worked().filter(|w| w.marked())?;
    let said = match maker {
        Some(name) => Msg::new(msg::describe::MARK).role("maker", Phrase::literal(name)),
        None => msg::describe::MARK_UNKNOWN.into(),
    };
    Some(line(LineKind::Stat, said))
}

/// `n` enchantments in words: "one enchantment", "two enchantments".
fn enchantments(n: usize) -> Msg {
    Msg::new(msg::describe::ENCHANTMENTS).role("n", num(n))
}

/// What a grimoire teaches: its spell and level, then
/// where `user` stands with it: a spell they know at its rank and that reading it would
/// deepen (`You know it at rank 3 of 10: reading it would deepen it.`), one known at the
/// last rank, or one they could learn. Nothing for anything else.
fn grimoire_lines(stack: &Stack, lore: Lore<'_>, user: Option<&Actor>) -> Vec<Line> {
    let content = lore.content;
    let Some(spell) = content.object(stack.object).grimoire else {
        return Vec::new();
    };
    let def = content.spell(spell);
    let teaches = (Msg::new(msg::describe::GRIMOIRE_TEACHES)
        .role("spell", content.spell_phrase(def)))
    .role("level", num(def.level))
    .role("about", Arg::said(data_line::spell_description(&def.id)));
    let mut out = vec![line(LineKind::Stat, teaches)];
    let most = crate::content::MAX_SPELL_RANK;
    let standing = match user.and_then(|u| u.spell_rank(content, spell)) {
        Some(rank) if rank < most => {
            (Msg::new(msg::describe::GRIMOIRE_KNOWN).role("n", num(rank))).role("most", num(most))
        }
        Some(_) => Msg::new(msg::describe::GRIMOIRE_KNOWN_ALL).role("most", num(most)),
        None => msg::describe::GRIMOIRE_UNKNOWN.into(),
    };
    out.push(line(LineKind::Note, standing));
    out
}

/// What a wand holds, as the player knows it: once its type is
/// known, its spell and what it does in `user`'s hands at the wand's rung (`Holds Fireball: 4d6
/// fire, radius 2`, [`wand_does`]), or the spell in words where it rolls nothing or nobody holds
/// it (`Holds Sleep (level 7): ...`), else that it is unknown; its charges once it is identified
/// (`7 charges left`), else that they are unknown. Nothing for anything else.
fn wand_lines(stack: &Stack, lore: Lore<'_>, user: Option<&Actor>) -> Vec<Line> {
    let content = lore.content;
    let Some(wand) = content.object(stack.object).wand else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if lore.is_identified(stack.object) {
        let spell = content.spell(wand.spell);
        let named = content.spell_phrase(spell);
        let said = match user.and_then(|u| wand_does(stack, lore, u)) {
            Some(does) => {
                (Msg::new(msg::describe::WAND_HOLDS).role("spell", named)).role("does", does)
            }
            None => (Msg::new(msg::describe::WAND_HOLDS_SPELL).role("spell", named))
                .role("level", num(spell.level))
                .role("about", Arg::said(data_line::spell_description(&spell.id))),
        };
        out.push(line(LineKind::Stat, said));
    } else {
        out.push(line(LineKind::Note, msg::describe::WAND_UNKNOWN));
    }
    out.push(match stack.known_charges(content) {
        Some(0) => line(LineKind::Stat, msg::describe::WAND_EMPTY),
        Some(n) => line(
            LineKind::Stat,
            Msg::new(msg::describe::WAND_CHARGES).role("n", num(n)),
        ),
        None => line(LineKind::Note, msg::describe::WAND_CHARGES_UNKNOWN),
    });
    out
}

/// What the wand `stack` does in `user`'s hands, once its type is known (the spell scaling design
/// §7.4: its rung comes with its type): its roll at its rung plus their wand rungs, their other
/// hooks applied, with the damage type or "healed", then a burst's radius as they cast it
/// (`4d6 fire, radius 2`), as the abilities menu shows a spell's. `None` before its type is
/// known, for anything else, and for a wand whose spell rolls nothing.
pub(crate) fn wand_does(stack: &Stack, lore: Lore<'_>, user: &Actor) -> Option<Msg> {
    let content = lore.content;
    let wand = content.object(stack.object).wand?;
    let rung = stack.known_wand_rung(lore)?;
    let spell = content.spell(wand.spell);
    let roll = user.spell_roll_words(content, spell, CastBy::Wand(rung))?;
    Some(match user.spell_reach(content, spell).radius {
        Some(radius) => (Msg::new(msg::describe::WAND_DOES_RADIUS).role("roll", roll.text))
            .role("radius", num(radius)),
        None => roll.text,
    })
}

/// What a rod holds, as the player knows it: its effect in words
/// and its cost once its type is known (`Holds Digging: ...`, `Costs 6 mana a use, before your
/// talents`), else that it is unknown. Nothing for anything else.
fn rod_lines(stack: &Stack, lore: Lore<'_>) -> Vec<Line> {
    let content = lore.content;
    let Some(rod) = content.object(stack.object).rod else {
        return Vec::new();
    };
    if !lore.is_identified(stack.object) {
        return vec![line(LineKind::Note, msg::describe::ROD_UNKNOWN)];
    }
    let effect = content.rod_effect(rod.effect);
    let holds = (Msg::new(msg::describe::ROD_HOLDS)
        .role("effect", Arg::said(data_line::rod_effect_name(&effect.id))))
    .role(
        "about",
        Arg::said(data_line::rod_effect_description(&effect.id)),
    );
    let costs = Msg::new(msg::describe::ROD_COSTS).role("mana", num(rod.mana));
    vec![line(LineKind::Stat, holds), line(LineKind::Stat, costs)]
}

/// What a potion does, as the player knows it: in words once its type is
/// known ("Heals wounds at once"), never its dice; else that it is unknown. Nothing for anything
/// else.
fn potion_lines(stack: &Stack, lore: Lore<'_>) -> Vec<Line> {
    let content = lore.content;
    let Some(drink) = &content.object(stack.object).drink else {
        return Vec::new();
    };
    if !lore.is_identified(stack.object) {
        return vec![line(LineKind::Note, msg::describe::POTION_UNKNOWN)];
    }
    // What its cures end, by the statuses' names ("bleeding").
    let cured: Vec<Arg> = (content.status_ids())
        .filter(|(_, s)| {
            (drink.does.cure_means().iter()).any(|c| s.cured_by.iter().any(|b| b == c))
        })
        .map(|(id, _)| content.status_phrase(id).into())
        .collect();
    let none = cured.is_empty();
    let cures = Arg::List(cured);
    let said = match &drink.does {
        Draught::Heal { .. } if none => msg::describe::POTION_HEAL.into(),
        Draught::Heal { .. } => Msg::new(msg::describe::POTION_HEAL_CURES).role("cures", cures),
        Draught::Cure { diseases: true, .. } if none => msg::describe::POTION_CURE_DISEASES.into(),
        Draught::Cure { diseases: true, .. } => {
            Msg::new(msg::describe::POTION_CURE_DISEASES_AND).role("cures", cures)
        }
        Draught::Cure { .. } => Msg::new(msg::describe::POTION_CURE).role("cures", cures),
        Draught::Mana { .. } => msg::describe::POTION_MANA.into(),
        Draught::Elixir { stat, .. } => {
            Msg::new(msg::describe::POTION_ELIXIR).role("stat", stat.comparative())
        }
        Draught::Harm { .. } => msg::describe::POTION_HARM.into(),
        Draught::Boost { status, severity } => {
            let def = content.status(*status);
            let raised: Vec<Arg> = (def.penalty.iter())
                .flat_map(|p| p.by.iter())
                .filter(|&(_, v)| v > 0)
                .map(|(stat, _)| boosted(stat).into())
                .collect();
            let much = if *severity > 1 {
                msg::describe::POTION_BOOST_MUCH
            } else {
                msg::describe::POTION_BOOST
            };
            Msg::new(much).role("raised", Arg::List(raised))
        }
        Draught::Vision { status } => {
            let def = content.status(*status);
            let eyes = (def.vision).map(|i| &content.vision_profiles()[i]);
            match eyes {
                None if def.sees_unseen => msg::describe::POTION_UNSEEN,
                Some(e) if e.sees_uv => msg::describe::POTION_UV,
                Some(e) if e.darkvision > 0 => msg::describe::POTION_DARK,
                _ => msg::describe::POTION_SHARPEN,
            }
            .into()
        }
        Draught::Haste { .. } => msg::describe::POTION_HASTE.into(),
        Draught::Resistance { status } => {
            let kinds: Vec<Arg> = (content.status(*status).resist.iter())
                .map(|&(kind, _)| content.damage_type_phrase(kind).into())
                .collect();
            Msg::new(msg::describe::POTION_WARD).role("kinds", Arg::List(kinds))
        }
        Draught::Nourish { .. } => msg::describe::POTION_NOURISH.into(),
        Draught::Restore => msg::describe::POTION_RESTORE.into(),
        Draught::Blink => msg::describe::POTION_BLINK.into(),
        Draught::Status { status, .. } => {
            Msg::new(msg::describe::POTION_STATUS).role("does", lent(content, *status))
        }
        Draught::Gamble { .. } => msg::describe::POTION_GAMBLE.into(),
    };
    let mut lines = vec![line(LineKind::Stat, said)];
    // A flask: what its splash is, by its spell's name.
    if let Some(flask) = &content.object(stack.object).flask {
        let spell = content.spell_phrase(content.spell(flask.spell));
        lines.push(line(
            LineKind::Stat,
            Msg::new(msg::describe::FLASK).role("spell", spell),
        ));
    }
    lines
}

/// What a status draught's status does for its drinker, in words, by the first thing it does
/// that the words know ("Makes you harder to hit"), else by its name ("Makes you barkskin").
fn lent(content: &Content, status: crate::content::StatusId) -> Msg {
    let def = content.status(status);
    let does = if def.unseen {
        msg::describe::POTION_STATUS_HIDES
    } else if def.wades_free || def.ice_holds {
        msg::describe::POTION_STATUS_WATER
    } else if def.senses.is_some() {
        msg::describe::POTION_STATUS_FEEL
    } else if def.regen > 100 {
        msg::describe::POTION_STATUS_WOUNDS
    } else if def.combat.dodge > 0 || def.combat.parry > 0 || def.combat.shield > 0 {
        msg::describe::POTION_STATUS_HARDER
    } else if def.combat.to_hit > 0 || def.combat.damage > 0 {
        msg::describe::POTION_STATUS_SURER
    } else if def.cost < 100 {
        msg::describe::POTION_STATUS_QUICKENS
    } else {
        return Msg::new(msg::describe::POTION_STATUS_NAMED)
            .role("status", content.status_phrase(status));
    };
    does.into()
}

/// What a boost of `stat` makes its drinker, in words: "stronger".
fn boosted(stat: crate::stats::Stat) -> MessageId {
    use crate::stats::Stat;
    match stat {
        Stat::Strength => msg::describe::STAT_STRENGTH,
        Stat::Dexterity => msg::describe::STAT_DEXTERITY,
        Stat::Toughness => msg::describe::STAT_TOUGHNESS,
        Stat::Intelligence => msg::describe::STAT_INTELLIGENCE,
        Stat::Willpower => msg::describe::STAT_WILLPOWER,
        Stat::Presence => msg::describe::STAT_PRESENCE,
    }
}

/// The description of `stack`, as the player knows it: what it is (`Magic long sword · steel
/// blade, leather grip · 1.8 kg`: every part, as materials show identified or not), each gem in
/// it with its size and cut (`Pommel: small emerald, fine cut`; a loose gem's `Gem: tiny
/// garnet, uncut`), its base stats, and its affixes if identified (`Sharp: +2 damage`) or a note
/// that its magic is unknown.
pub fn lines(stack: &Stack, content: &Content) -> Vec<Line> {
    lines_named(stack, content, stack.made_phrase(content), None)
}

/// [`lines`] with what one of it is called (a scroll's label), and whose mark it bears, if
/// marked.
fn lines_named(stack: &Stack, content: &Content, name: Phrase, maker: Option<&str>) -> Vec<Line> {
    let def = content.object(stack.object);
    let mut out = Vec::new();
    let thing: Arg = if stack.count == 1 {
        let word = stack.worked().and_then(super::Worked::word);
        super::name::ahead(word, name).into()
    } else {
        (Msg::new(msg::describe::BASE_COUNT).role("n", num(stack.count)))
            .role("what", name.count(stack.count))
            .into()
    };
    let what: Arg = match stack.rarity() {
        Rarity::Normal => thing,
        rarity => (Msg::new(msg::describe::BASE_RARITY).role("rarity", rarity.word()))
            .role("what", thing)
            .into(),
    };
    let parts = stack.parts(content);
    let weight = format_weight(stack.weight(content));
    let base = (Msg::new(msg::describe::BASE).role("what", what))
        .role("made", parts_arg(content, &parts))
        .role("weight", Arg::notation(weight));
    out.push(line(LineKind::Base, base));
    for part in parts.iter() {
        let Some(cutting) = part.gem else { continue };
        let gem = crate::gem::phrase(content, part.material, cutting);
        let said = match part.part {
            Some(p) => {
                let kind = Phrase::id(format!("part:{}", content.part_defs()[usize::from(p.0)].id));
                (Msg::new(msg::describe::GEM).role("part", kind)).role("gem", gem)
            }
            None => Msg::new(msg::describe::GEM_LOOSE).role("gem", gem),
        };
        out.push(line(LineKind::Base, said));
    }
    // Its own stats as it is made: another material's damage and Protection (#193), the dice an
    // anvil's work left it; a spirit weapon's with its gifts' threat and piercing.
    let gift = crate::world::spirit_gift(content).filter(|_| stack.is_spirit());
    let spirit = gift.map(|g| Spirit {
        threat: g.threat,
        pierces: g.pierces,
    });
    let (_, damage) = crate::variant::weapon(def, &parts, content);
    let protection = stack.base_protection(content);
    let shield = def.shield + crate::variant::shield(def, &parts, content);
    let attack = stack.attack(content);
    out.extend(
        stats(
            def,
            attack.as_ref(),
            content,
            damage,
            (protection, shield),
            spirit,
        )
        .into_iter()
        .map(|t| line(LineKind::Stat, t)),
    );
    // Then what it gives in all, in numbers, as the player knows it (`numbers`): to hit and
    // damage with its material and known affixes, Dv and Pv.
    let numbers = super::Numbers::of(stack, content).lines(content);
    out.extend(numbers.into_iter().map(|t| line(LineKind::Stat, t)));
    out.extend(mark_line(stack, maker));
    // A spirit weapon's power, as a unique's is said: the data's words.
    if let Some(gift) = gift {
        let said = data_line::said(data_line::spirit_weapon_phrase(gift.talent));
        out.push(line(LineKind::Power, said));
    }
    if stack.rarity() == Rarity::Normal {
        return out;
    }
    // A curse the player knows of (#1754), identified or shown by wearing or throwing it.
    if stack.known_cursed(content) {
        out.push(line(LineKind::Note, msg::describe::CURSED));
    }
    match stack.known_magic() {
        None => out.push(line(LineKind::Note, msg::describe::MAGIC_UNKNOWN)),
        Some(magic) => {
            let kind = content.item_kind(stack).unwrap_or(ItemKind::Ring);
            for rolled in magic.rolled(content) {
                let said = affix_line(&rolled, Some(kind), content);
                out.push(line(LineKind::Affix(rolled.def.kind), said));
            }
            // A unique item's power and lore: the data's words.
            if let Some(unique) = magic.unique.as_deref().and_then(|id| content.unique(id)) {
                if unique.phrase.is_some() {
                    let said = data_line::said(data_line::unique_phrase(&unique.id));
                    out.push(line(LineKind::Power, said));
                }
                // A Scars power's scars (`scars.rs`), counted, never what each gave.
                if let Some(n) = stack.scars().map(|s| s.count()).filter(|&n| n > 0) {
                    let said = Msg::new(msg::uniques::SCARS).role("n", num(n));
                    out.push(line(LineKind::Note, said));
                }
                if unique.lore.is_some() {
                    let said = data_line::said(data_line::unique_lore(&unique.id));
                    out.push(line(LineKind::Lore, said));
                }
            }
        }
    }
    out
}

/// An affix's line: its name and what it does on an item of `kind` ([`phrase`]): "Sharp: +2
/// damage"; with no kind, its name alone before the colon. The enchanting choice's too.
pub fn affix_line(rolled: &Rolled<'_>, kind: Option<ItemKind>, content: &Content) -> Msg {
    let does: Arg = kind.map_or_else(Arg::empty, |k| phrase(rolled, k, content).into());
    (Msg::new(msg::describe::AFFIX).role("affix", Phrase::id(format!("affix:{}", rolled.def.id))))
        .role("does", does)
}

/// The creatures a bane affix is the bane of, said `plural`: `orcs`, `undead`.
fn bane_of(rolled: &Rolled<'_>) -> Phrase {
    let tag = rolled.def.tag.as_deref().unwrap_or("creature");
    Phrase::id(format!("tag:{tag}"))
}

/// What `rolled`, on an item of `kind`, does, in plain language (a `describe.does_*` message):
/// "+2 damage", "adds 1d4 damage", "+1 to Strength", "attacks faster".
pub fn phrase(rolled: &Rolled<'_>, kind: ItemKind, content: &Content) -> Msg {
    let effect = rolled.def.effect;
    let v = rolled.value;
    if rolled.def.cursed {
        return hindrance(effect, v);
    }
    let local = effect.is_local_on(kind);
    let every = |id: MessageId| {
        let said = Msg::new(id).role("v", num(v));
        if local {
            said
        } else {
            Msg::new(msg::describe::DOES_WITH_EVERY_ATTACK).role("does", said)
        }
    };
    let by = |id: MessageId| Msg::new(id).role("v", num(v));
    // A dice family's value is its dice's sides, its tier's `dice` how many: `1d8`, `2d10`.
    let dice = |id: MessageId| {
        let count = rolled.dice;
        Msg::new(id).role("dice", Arg::notation(format!("{count}d{v}")))
    };
    match effect {
        Effect::ToHit => every(msg::describe::DOES_TO_HIT),
        Effect::Damage => every(msg::describe::DOES_DAMAGE),
        Effect::ExtraDamage => dice(msg::describe::DOES_EXTRA_DAMAGE),
        Effect::Elemental => {
            dice(msg::describe::DOES_ELEMENTAL).role("kind", kind_name(rolled, content))
        }
        Effect::Resistance => {
            by(msg::describe::DOES_RESISTANCE).role("kind", kind_name(rolled, content))
        }
        Effect::Threat => every(msg::describe::DOES_THREAT),
        Effect::Crit => every(msg::describe::DOES_CRIT),
        Effect::AttackSpeed if local => {
            Msg::new(msg::describe::DOES_ATTACK_SPEED).role("speed", faster(v))
        }
        Effect::AttackSpeed => {
            Msg::new(msg::describe::DOES_ATTACK_SPEED_ALL).role("speed", faster(v))
        }
        Effect::Range => by(msg::describe::DOES_RANGE),
        Effect::Draw => by(msg::describe::DOES_DRAW),
        Effect::Durability => by(msg::describe::DOES_DURABILITY),
        Effect::Bane => by(msg::describe::DOES_BANE).role("bane", bane_of(rolled)),
        Effect::Parry => by(msg::describe::DOES_PARRY),
        // A unique's penalty (#2115: Hǣrsceorp's −1).
        Effect::Armour if v < 0 => Msg::new(msg::describe::DOES_ARMOUR_LESS).role("v", num(-v)),
        Effect::Armour => by(msg::describe::DOES_ARMOUR),
        Effect::Protection => by(msg::describe::DOES_PROTECTION),
        Effect::Shield => by(msg::describe::DOES_SHIELD),
        Effect::MaxDex => by(msg::describe::DOES_MAX_DEX),
        // A unique's penalty (#2127: the Kettle Hat's, a heavy pot).
        Effect::Weight if v < 0 => Msg::new(msg::describe::DOES_HEAVIER).role("v", num(-v)),
        Effect::Weight => by(msg::describe::DOES_LIGHTER),
        Effect::Proof => msg::describe::DOES_PROOF.into(),
        // A unique's penalty (#2091: the Grandmother's −2).
        Effect::Dodge if v < 0 => Msg::new(msg::describe::DOES_DODGE_LESS).role("v", num(-v)),
        Effect::Dodge => by(msg::describe::DOES_DODGE),
        Effect::Deflection => by(msg::describe::DOES_DEFLECTION),
        Effect::Natural => by(msg::describe::DOES_NATURAL),
        Effect::MaxHp => by(msg::describe::DOES_MAX_HP),
        Effect::Regeneration => by(msg::describe::DOES_REGENERATION),
        Effect::MaxMana => by(msg::describe::DOES_MAX_MANA),
        Effect::ManaRegeneration => by(msg::describe::DOES_MANA_REGENERATION),
        Effect::ManaCost => by(msg::describe::DOES_MANA_COST),
        Effect::SpellEase => by(msg::describe::DOES_SPELL_EASE),
        Effect::SpellToHit => by(msg::describe::DOES_SPELL_TO_HIT),
        Effect::SpellThreat => by(msg::describe::DOES_SPELL_THREAT),
        Effect::SpellRungs => by(msg::describe::DOES_SPELL_RUNGS),
        Effect::SpellBonus => by(msg::describe::DOES_SPELL_BONUS),
        Effect::SpellSides => by(msg::describe::DOES_SPELL_SIDES),
        Effect::SpellDice => by(msg::describe::DOES_SPELL_DICE),
        Effect::SpellReroll => by(msg::describe::DOES_SPELL_REROLL),
        Effect::SpellReach => by(msg::describe::DOES_SPELL_REACH),
        Effect::SpellDuration => by(msg::describe::DOES_SPELL_DURATION),
        Effect::ManaOnKill => by(msg::describe::DOES_MANA_ON_KILL),
        Effect::WandRungs => by(msg::describe::DOES_WAND_RUNGS),
        Effect::ResourceMax => {
            by(msg::describe::DOES_RESOURCE_MAX).role("pool", pool_name(rolled, content))
        }
        Effect::ResourceRate => {
            by(msg::describe::DOES_RESOURCE_RATE).role("pool", pool_name(rolled, content))
        }
        Effect::ActiveStamina => {
            by(msg::describe::DOES_STAMINA).role("pool", pool_name(rolled, content))
        }
        Effect::ResourceOnKill => {
            by(msg::describe::DOES_RESOURCE_ON_KILL).role("pool", pool_name(rolled, content))
        }
        Effect::Stat(stat) => by(msg::describe::DOES_STAT).role("stat", stat.name()),
        Effect::MoveSpeed => Msg::new(msg::describe::DOES_MOVE_SPEED).role("speed", faster(v)),
        Effect::Carry => by(msg::describe::DOES_CARRY),
        Effect::Stealth => by(msg::describe::DOES_STEALTH),
        Effect::DoorKick => msg::describe::DOES_DOOR_KICK.into(),
        Effect::DigPercent => by(msg::describe::DOES_DIG),
        Effect::Hunger => by(msg::describe::DOES_HUNGER),
        Effect::TrapDamage => by(msg::describe::DOES_TRAP_DAMAGE),
        Effect::DiseaseCheck => by(msg::describe::DOES_DISEASE_CHECK),
        Effect::PotionKept => by(msg::describe::DOES_POTION_KEPT),
        Effect::Light => {
            let radius = (rolled.light).map_or(0, |id| content.light(id).radius);
            match radius {
                ..=2 => msg::describe::DOES_GLOWS_FAINTLY,
                3 => msg::describe::DOES_GLOWS,
                _ => msg::describe::DOES_GLOWS_BRIGHTLY,
            }
            .into()
        }
        Effect::Skill => match rolled.def.skill {
            Some(id) => by(msg::describe::DOES_SKILL).role("skill", content.skill_phrase(id)),
            None => by(msg::describe::DOES_SKILL_ANY),
        },
    }
}

/// What a curse's effect `effect` of value `v` does (#1754): the effect the other way.
fn hindrance(effect: Effect, v: i32) -> Msg {
    match effect {
        Effect::Weight => Msg::new(msg::describe::DOES_HEAVIER).role("v", num(v)),
        Effect::MoveSpeed => Msg::new(msg::describe::DOES_MOVE_SPEED).role("speed", slower(v)),
        Effect::Hunger => Msg::new(msg::describe::DOES_HUNGER_FASTER).role("v", num(v)),
        _ => Msg::new(msg::describe::DOES_HINDERS).role("effect", Arg::id(effect.id())),
    }
}

/// How much slower, in words: speed numbers stay hidden.
fn slower(percent: i32) -> MessageId {
    match percent {
        ..=7 => msg::describe::SPEED_SLOWER_LITTLE,
        8..=15 => msg::describe::SPEED_SLOWER,
        _ => msg::describe::SPEED_SLOWER_MUCH,
    }
}

/// The damage type a typed affix names ("fire"); "some kind of" for one that names none.
fn kind_name(rolled: &Rolled<'_>, content: &Content) -> Arg {
    (rolled.def.damage_type).map_or_else(
        || msg::describe::SOME_KIND.into(),
        |k| content.damage_type_phrase(k).into(),
    )
}

/// The resource a pooled affix names, in lower case ("stamina"); "a pool" for one that names
/// none.
fn pool_name(rolled: &Rolled<'_>, content: &Content) -> Arg {
    (rolled.def.resource).map_or_else(
        || msg::describe::A_POOL.into(),
        |id| content.resource_phrase(id).into(),
    )
}

/// How much faster, in words: speed numbers stay hidden.
fn faster(percent: i32) -> MessageId {
    match percent {
        ..=7 => msg::describe::SPEED_FASTER_LITTLE,
        8..=12 => msg::describe::SPEED_FASTER,
        _ => msg::describe::SPEED_FASTER_MUCH,
    }
}

/// The base stats of `def` in words: how it is held or worn, its damage and criticals, how far it
/// shoots or is thrown, what it needs, how quick it is, its Defence, and whether it breaks.
pub fn base_stats(def: &ObjectDef, content: &Content) -> Vec<Msg> {
    stats(
        def,
        def.attack.as_ref(),
        content,
        0,
        (def.protection, def.shield),
        None,
    )
}

/// What a spirit weapon's gifts change in its own stats: the threat lowered, and whether it
/// pierces armour.
#[derive(Clone, Copy)]
struct Spirit {
    threat: u8,
    pierces: bool,
}

/// [`base_stats`] of one made of another material: `damage` more to its damage dice (its dice
/// are `attack`'s), and `protection` its Protection and `shield` its Defence as a shield.
fn stats(
    def: &ObjectDef,
    attack: Option<&crate::combat::Attack>,
    content: &Content,
    damage: i32,
    (protection, shield): (i32, i32),
    spirit: Option<Spirit>,
) -> Vec<Msg> {
    let mut out = Vec::new();
    let n = |id: MessageId, n: i32| Msg::new(id).role("n", num(n));
    if let Some(held) = how_held(def, content) {
        out.push(held);
    }
    if let Some(attack) = attack {
        let less = spirit.map_or(0, |s| i32::from(s.threat));
        let threat = (i32::from(attack.threat) - less).max(crate::combat::FUMBLE + 1);
        let critical = if threat >= HIGHEST {
            format!("{HIGHEST}")
        } else {
            format!("{threat}–{HIGHEST}")
        };
        let id = if def.launches.is_some() {
            msg::describe::DAMAGE_SHOT
        } else {
            msg::describe::DAMAGE
        };
        out.push(
            (Msg::new(id).role(
                "dice",
                Arg::notation(attack.damage.plus(damage).to_string()),
            ))
            .role("critical", Arg::notation(critical))
            .role("crit", num(attack.crit)),
        );
        // Its kinds of damage: "Slashing or piercing damage".
        let kinds: Vec<Arg> = (attack.kinds.iter())
            .map(|k| content.damage_type_phrase(k).into())
            .collect();
        if let Some(kinds) = joined(kinds, msg::describe::OR_JOINED) {
            out.push(Msg::new(msg::describe::DAMAGE_KINDS).role("kinds", kinds));
        }
        if let Some(strength) = attack.strength {
            out.push(n(msg::describe::NEEDS_STRENGTH, i32::from(strength)));
        }
        if def.attack_cost.is_some() || def.launches.is_some() {
            let action = if def.launches.is_some() {
                ActionKind::Shoot
            } else {
                ActionKind::Melee
            };
            let normal = content.action_costs().cost(action);
            let cost = def.attack_cost.unwrap_or(normal);
            if cost < normal {
                out.push(msg::describe::QUICK.into());
            } else if cost > normal {
                out.push(msg::describe::SLOW.into());
            }
        }
    }
    match (def.range, def.launches.is_some()) {
        (Some(range), true) => out.push(n(msg::describe::SHOOTS, range)),
        (Some(range), false) => out.push(n(msg::describe::THROWN, range)),
        _ => {}
    }
    // A digging tool's power, its base stat as damage is a weapon's.
    if let Some(dig) = def.dig {
        out.push(Msg::new(msg::describe::DIGS).role("dig", num(dig)));
    }
    if def.reloads {
        out.push(msg::describe::RELOADS.into());
    }
    if let Some(added) = &def.added {
        let kind = content.damage_type_phrase(added.kind);
        out.push(
            (Msg::new(msg::describe::ADDS).role("dice", Arg::notation(added.dice.to_string())))
                .role("kind", kind),
        );
    }
    // How much armour it ignores stays hidden, as the soak does: only that it does.
    if def.penetration > 0 || spirit.is_some_and(|s| s.pierces) {
        out.push(msg::describe::PIERCES.into());
    }
    if def.parry > 0 {
        out.push(n(msg::describe::PARRY, def.parry));
    }
    if def.armour > 0 {
        out.push(n(msg::describe::ARMOUR, def.armour));
    }
    if protection > 0 {
        let against: Vec<Arg> = (def.protects.iter())
            .map(|&(k, more)| {
                let id = if more < 0 {
                    msg::describe::PROTECTS_LESS
                } else {
                    msg::describe::PROTECTS_MORE
                };
                n(id, more.abs())
                    .role("kind", content.damage_type_phrase(k))
                    .into()
            })
            .collect();
        out.push(match joined(against, msg::describe::LISTED) {
            None => n(msg::describe::PROTECTION, protection),
            Some(against) => {
                n(msg::describe::PROTECTION_AGAINST, protection).role("against", against)
            }
        });
    }
    let resist = |weak: bool| -> Option<Arg> {
        let kinds: Vec<Arg> = (def.resist.iter())
            .filter(|&&(_, p)| if weak { p < 0 } else { p > 0 })
            .map(|&(k, p)| {
                n(msg::describe::RESIST, p.abs())
                    .role("kind", content.damage_type_phrase(k))
                    .into()
            })
            .collect();
        joined(kinds, msg::describe::LISTED)
    };
    if let Some(kinds) = resist(false) {
        out.push(Msg::new(msg::describe::RESISTS).role("kinds", kinds));
    }
    if let Some(kinds) = resist(true) {
        out.push(Msg::new(msg::describe::VULNERABLE).role("kinds", kinds));
    }
    if shield > 0 {
        out.push(n(msg::describe::SHIELD, shield));
    }
    if let Some(max) = def.max_dex {
        out.push(n(msg::describe::MAX_DEX, max));
    }
    if def.breaks > 0 {
        out.push(n(msg::describe::BREAKS, i32::from(def.breaks)));
    }
    if def.attack.is_some_and(|attack| attack.finesse) {
        out.push(msg::describe::FINESSE.into());
    }
    out
}

/// How it is held or worn: "Long blade, for the main hand", "Bow: readied as a missile weapon,
/// shoots arrows", "Worn on the head".
fn how_held(def: &ObjectDef, content: &Content) -> Option<Msg> {
    let slot = def.slot.as_ref()?;
    let slot = content.slots().iter().find(|s| &s.fits == slot)?;
    let noun = weapon_noun_phrase(def, content);
    let place = Phrase::id(format!("slot:{}", slot.id));
    Some(match (noun, slot.action) {
        (Some(c), ActionKind::Wield) if def.two_handed => {
            Msg::new(msg::describe::HELD_BOTH_HANDS).role("noun", c)
        }
        (Some(c), ActionKind::Wield) => {
            (Msg::new(msg::describe::HELD_MAIN).role("noun", c)).role("slot", place)
        }
        (Some(c), _) if def.launches.is_some() => {
            let ammo: Arg = (def.launches.as_deref())
                .and_then(|kind| content.ammunition(kind))
                .map_or_else(
                    || msg::describe::AMMUNITION.into(),
                    |o| Phrase::id(format!("object:{}", o.id)).into(),
                );
            (Msg::new(msg::describe::AMMO_LAUNCHER).role("noun", c)).role("ammo", ammo)
        }
        (_, _) if def.ammo.is_some() => {
            let nouns: Vec<Phrase> = (content.object_defs().iter())
                .filter(|o| o.launches.is_some() && o.launches == def.ammo)
                .filter_map(|o| weapon_noun_phrase(o, content))
                .fold(Vec::new(), |mut all, c| {
                    if !all.contains(&c) {
                        all.push(c);
                    }
                    all
                });
            let from: Vec<Arg> = (nouns.into_iter()).map(|c| c.count(2).into()).collect();
            let from = joined(from, msg::describe::AND_JOINED).unwrap_or_else(Arg::empty);
            Msg::new(msg::describe::AMMO_QUIVERED).role("from", from)
        }
        (_, ActionKind::Wield) => Msg::new(msg::describe::HELD_IN).role("slot", place),
        (_, ActionKind::Ready) => Msg::new(msg::describe::READIED).role("slot", place),
        _ if slot.fits == "ring" => msg::describe::WORN_FINGER.into(),
        _ => Msg::new(msg::describe::WORN).role("slot", place),
    })
}

/// What the craftsperson's Eye for the make reads of `stack` (the
/// craftsperson design §7.2), in words and never a number, as `lore` names things: what salvage
/// would give of one (or of as many as salvage takes at once, "enough of them") if every check
/// went well, at `percent` of each material's grams, a gem whole
/// ("Taken apart, it would give 2 steel ingots and a small ruby."; "Nothing in it is worth
/// saving."; "It would not come apart." for one `protected` from harm); for arms or armour an
/// anvil or a knife betters, the improvements it will still take, or that it is damaged (never
/// the attempts made on it, the difficulty's secret; "or your own hands" to one who `mends`, a
/// holder of Make do and mend); and how hard its main material is to work,
/// when it is hard or soft (its `work`: +4 or more very hard, +1 to +3 hard, -2 or less soft and
/// easy). Perception only: nothing is drawn.
pub fn make_lines(
    stack: &Stack,
    lore: Lore<'_>,
    percent: u8,
    protected: bool,
    mends: bool,
) -> Vec<Line> {
    let content = lore.content;
    let mut out = Vec::new();
    let worked = |s: &Stack| {
        (s.parts(content).first())
            .is_some_and(|p| content.stock().profession_of(content, p.material).is_some())
    };
    // Salvage takes as many as make a piece (#1777): "enough of them", never the number.
    let batch = (crate::world::salvageable(content, stack))
        .then(|| crate::piece::salvage_batch(stack, content, percent))
        .flatten()
        .map_or(1, |(n, _)| n);
    let mut some = stack.clone();
    some.count = batch;
    let saved: Vec<Stack> = match crate::world::salvageable(content, &some) {
        true => crate::piece::salvage(&some, content, percent)
            .into_iter()
            .filter(worked)
            .collect(),
        false => Vec::new(),
    };
    let apart: Msg = match () {
        () if protected => msg::describe::MAKE_PROTECTED.into(),
        () if saved.is_empty() => msg::describe::MAKE_NOTHING.into(),
        () if batch == 1 => Msg::new(msg::describe::MAKE_APART)
            .role("what", crate::world::items::stacks_arg(&saved, lore)),
        () => Msg::new(msg::describe::MAKE_APART_BATCH)
            .role("what", crate::world::items::stacks_arg(&saved, lore)),
    };
    out.push(line(LineKind::Note, apart));
    let place = match () {
        () if crate::world::temperable(content, stack) => Some(msg::describe::PLACE_ANVIL),
        () if crate::world::whittleable(content, stack) => Some(msg::describe::PLACE_KNIFE),
        () => None,
    };
    if let Some(place) = place {
        let most = usize::from(content.temper().most);
        let worked = stack.worked();
        let said: Msg = match worked.map_or(0, |w| w.improved.len()) {
            _ if worked.is_some_and(|w| w.damaged) => {
                let id = if mends {
                    msg::describe::MAKE_DAMAGED_MEND
                } else {
                    msg::describe::MAKE_DAMAGED
                };
                Msg::new(id).role("place", place)
            }
            done if done >= most => msg::describe::MAKE_FINEST.into(),
            // `most` is at most the grades' three (`data/rules/temper.toml`).
            done => (Msg::new(msg::describe::MAKE_IMPROVEMENTS)
                .role("n", num((most - done).min(3))))
            .role("place", place),
        };
        out.push(line(LineKind::Note, said));
    }
    if let Some(part) = stack.parts(content).first() {
        let material = content.material(part.material);
        let hard = match material.work {
            w if w >= 4 => Some(msg::describe::HARD_VERY),
            1..=3 => Some(msg::describe::HARD),
            w if w <= -2 => Some(msg::describe::HARD_SOFT),
            _ => None,
        };
        if let Some(hard) = hard {
            let name = Phrase::id(format!("material:{}", material.id));
            out.push(line(
                LineKind::Note,
                (Msg::new(msg::describe::MAKE_HARD).role("material", name)).role("hard", hard),
            ));
        }
    }
    out
}
