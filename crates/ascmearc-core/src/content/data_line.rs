//! The narrator's lines of data (#1944; `docs/dev/localization.md` "Lines of data", whose table
//! lists every key): each is said by its key, never its text, and English is the data itself,
//! added here by key once the content is loaded ([`add_all`]). A key is the entity's kind, its
//! id, and the field's path in its file (`status:bleeding.start`, `creature:troll.on_eat.message`,
//! `level:ceaptun.arrival`); a rules file's line is the file's stem and the line's path in it
//! (`hunger:corpses.skills.spellcraft.learn`, `familiars:arrives.cat`, `laws:eat.first`), a row
//! of its arrays named by the id that tells it apart (`craving:threshold.<talent>.message`). Each
//! function here gives one key, so the loader and the `say` site agree. None of these lines names
//! a role but a name kept in a save ([`saved_names`]: a shop's sign names its `keeper`, a burial
//! ground's epitaph its dead): a fragment (a charm's `message`, a creature's `depart`, a
//! familiar's arrival) is said inside a template of `messages/allies.toml`, which names the one
//! it is about.

use super::{Content, ContentError, StatusDef, TalentDef, TalentEffect, TalentId};
use super::{OutcomeDef, RoomEffectDef};
use crate::alignment::Band;
use crate::lang::{Arg, Line, Msg, Template};
use crate::talent::SlotEffect;
use crate::talent::{MovementEffect, WarmthBand};

/// The line of data at `key`, said alone and naming no one.
pub fn said(key: String) -> Msg {
    Msg::data_line(Arg::said(key))
}

/// A status's `start` line, or its first stage where `stages` gives it (the data gives one or
/// the other).
pub fn status_start(def: &StatusDef) -> String {
    match def.stages.is_empty() {
        true => format!("status:{}.start", def.id),
        false => format!("status:{}.stages.0", def.id),
    }
}

/// A status's `end` line.
pub fn status_end(def: &StatusDef) -> String {
    format!("status:{}.end", def.id)
}

/// A status's line for degree `degree` (1 and up), its `stages[degree - 1]`.
pub fn status_stage(def: &StatusDef, degree: usize) -> String {
    format!("status:{}.stages.{}", def.id, degree.saturating_sub(1))
}

/// A status's Heal line: its `treat.made`, else its `treat.failed`.
pub fn status_treat(def: &StatusDef, made: bool) -> String {
    let field = if made { "made" } else { "failed" };
    format!("status:{}.treat.{field}", def.id)
}

/// A disease's `recovered` line.
pub fn disease_recovered(id: &str) -> String {
    format!("disease:{id}.recovered")
}

/// What eating creature `id`'s fresh corpse says (`on_eat.message`).
pub fn on_eat(id: &str) -> String {
    format!("creature:{id}.on_eat.message")
}

/// Creature `id`'s departure as an ally whose time ran out ("fades away"), its `depart`.
pub fn depart(id: &str) -> String {
    format!("creature:{id}.depart")
}

/// What a unique item `id` tells its wearer as its power works (`power.message`): a Ward's as it
/// wards off a status, a Negate's as a hit comes to nothing.
pub fn ward(id: &str) -> String {
    format!("unique:{id}.power.message")
}

/// A flask `id`'s words as it shatters (`throw.words`).
pub fn flask_words(id: &str) -> String {
    format!("object:{id}.throw.words")
}

/// A charm spell `id`'s words as it takes hold (`effect.charm.message`), or as it ends (`ends`).
pub fn charm(id: &str, ends: bool) -> String {
    let field = if ends { "ends" } else { "message" };
    format!("spell:{id}.effect.charm.{field}")
}

/// How a familiar of creature `id` arrives (`familiars.toml`'s `[arrives]`).
pub fn familiar_arrives(id: &str) -> String {
    format!("familiars:arrives.{id}")
}

/// What a corpse teaching skill `skill` says (`hunger.toml`'s `[corpses.skills]`): `learn` the
/// first time, else `rise`.
pub fn skill_gift(skill: &str, learn: bool) -> String {
    let field = if learn { "learn" } else { "rise" };
    format!("hunger:corpses.skills.{skill}.{field}")
}

/// Alignment band `band`'s `rise` (`rise` true) or `fall` line (`alignment.toml`'s `[[band]]`).
pub fn alignment_band(band: Band, rise: bool) -> String {
    let field = if rise { "rise" } else { "fall" };
    format!("alignment:band.{}.{field}", band.id())
}

/// The craving's line at `path` (`first`, `again.message`, `surrender.first`; `craving.toml`).
pub fn craving(path: &str) -> String {
    format!("craving:{path}")
}

/// The message of the craving's `[[threshold]]` row that gives `talent`.
pub fn craving_threshold(content: &Content, talent: TalentId) -> String {
    let talent = &content.talents().get(talent).id;
    format!("craving:threshold.{talent}.message")
}

/// Law `law`'s (`eat`, `summon`) `first` line (`laws.toml`).
pub fn law_first(law: &str) -> String {
    format!("laws:{law}.first")
}

/// The message of the oath's `[[band]]` that gives `talent` (`oaths.toml`).
pub fn oath_band(content: &Content, talent: TalentId) -> String {
    let talent = &content.talents().get(talent).id;
    format!("oaths:band.{talent}.message")
}

/// Profane gift `gift`'s `field` (`line`, `undone`, `instead`).
pub fn profane(gift: &str, field: &str) -> String {
    format!("profane:{gift}.{field}")
}

/// The `slower` (`slower` true) or `faster` line of band `band` (from 0) of talent `talent`'s
/// `temperature_speed` effect (cold blood).
pub fn warmth(talent: &str, band: usize, slower: bool) -> String {
    let field = if slower { "slower" } else { "faster" };
    format!("talent:{talent}.effects.temperature_speed.bands.{band}.{field}")
}

/// The bands of every `temperature_speed` of talent `def`, its `while` effects' too.
pub fn warmth_bands(def: &TalentDef) -> impl Iterator<Item = &[WarmthBand]> {
    let whiles = def.effects_while.iter().map(|(_, e)| e);
    (def.effects.iter().chain(whiles)).filter_map(|effect| match effect {
        TalentEffect::Movement(MovementEffect::TemperatureSpeed { bands }) => {
            Some(bands.as_slice())
        }
        _ => None,
    })
}

// Crafting (#1917).

/// Profession `skill`'s `fumbled` (`fumbled` true) or `failed` line (`recipes.toml`'s
/// `[[profession]]`).
pub fn profession(skill: &str, fumbled: bool) -> String {
    let field = if fumbled { "fumbled" } else { "failed" };
    format!("recipes:profession.{skill}.{field}")
}

/// Profession `skill`'s `verb` (`recipes.toml`'s `[[profession]]`): the craft dialog's verb
/// where no recipe names its own ("Brew").
pub fn profession_verb(skill: &str) -> String {
    format!("recipes:profession.{skill}.verb")
}

/// Recipe `id`'s own `verb`, the craft dialog's (a ring is made, not cut).
pub fn recipe_verb(id: &str) -> String {
    format!("recipe:{id}.verb")
}

/// The verb of stock group `group`'s profession (`stock.toml`'s `[[group]]`), or with `None`
/// the gems' (`gems`): "Forge", "Cut".
pub fn stock_verb(group: Option<&str>) -> String {
    match group {
        Some(group) => format!("stock:group.{group}.verb"),
        None => "stock:gems.verb".to_string(),
    }
}

/// Rig `id`'s `snaps` (`snaps` true) or `made` line (`rigs.toml`'s `[[rig]]`).
pub fn rig(id: &str, snaps: bool) -> String {
    let field = if snaps { "snaps" } else { "made" };
    format!("rigs:rig.{id}.{field}")
}

/// A hand-made level `id`'s `arrival` line.
fn level_arrival(id: &str) -> String {
    format!("level:{id}.arrival")
}

/// A branch `id`'s `arrival` line, said on its first level.
fn dungeon_arrival(id: &str) -> String {
    format!("dungeon:{id}.arrival")
}

/// The arrival line level `id` says ([`Content::arrival`]), if it has one: a hand-made level's
/// own, else its branch's on the branch's first level.
pub fn arrival(content: &Content, id: &str) -> Option<String> {
    if let Some(template) = content.levels.get(id) {
        return template.arrival.as_ref().map(|_| level_arrival(id));
    }
    match content.branch_level(id)? {
        (def, 1) => def.arrival.as_ref().map(|_| dungeon_arrival(&def.id)),
        _ => None,
    }
}

// The magic's lines (L4k, #1922): a spell's look, a miscast's and a mishap's message, a ruin
// row's words, and a reprieve's; a risen corpse's berserk line (L4l, #1923).

/// Spell `id`'s `[spell.cast]` `message` (its `look.message`), said as it is cast.
pub fn spell_cast(id: &str) -> String {
    format!("spell:{id}.cast.message")
}

/// Spell `id`'s `rise` `berserk_message`, said in place of its cast message when the corpse
/// rises berserk.
pub fn rise_berserk(id: &str) -> String {
    format!("spell:{id}.effect.rise.berserk_message")
}

/// Miscast `id`'s `message`, said to the player who miscast.
pub fn miscast_message(id: &str) -> String {
    format!("miscast:{id}.message")
}

/// Mishap `id`'s `message`, said as it happens.
pub fn mishap_message(id: &str) -> String {
    format!("mishap:{id}.message")
}

/// What `ruin.toml`'s row `row` says of the items it ruined: its `says[0]` of one (`many`
/// false), its `says[1]` of several. A row is named by its damage type and what it matches
/// (`ruin:fire.class.scrolls.says.0`, `ruin:cold.group.glass.says.1`).
pub fn ruin_says(content: &Content, row: &super::RuinRow, many: bool) -> String {
    let kind = &content.damage_type(row.kind).id;
    let of = match &row.of {
        super::RuinOf::Class(c) => format!("class.{}", content.item_class(*c).id),
        super::RuinOf::Material(m) => format!("material.{}", content.material(*m).id),
        super::RuinOf::Group(g) => format!("group.{g}"),
    };
    format!("ruin:{kind}.{of}.says.{}", u8::from(many))
}

/// The `message` of talent `talent`'s `reprieve` effect (Miracle, Hag's bargain).
pub fn reprieve(talent: &str) -> String {
    format!("talent:{talent}.effects.reprieve.message")
}

/// The reprieves of talent `def`, its `while` effects' too: each its `once` and its `message`.
pub fn reprieves(def: &TalentDef) -> impl Iterator<Item = (&str, &str)> {
    use crate::talent::VigourEffect;
    let whiles = def.effects_while.iter().map(|(_, e)| e);
    (def.effects.iter().chain(whiles)).filter_map(|effect| match effect {
        TalentEffect::Vigour(VigourEffect::Reprieve { once, message, .. }) => {
            Some((once.as_str(), message.as_str()))
        }
        _ => None,
    })
}

/// The magic's lines above, for [`add_all`].
fn magic_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for spell in content.spells() {
        if let Some(message) = &spell.look.message {
            add(spell_cast(&spell.id), message);
        }
        if let Some(rise) = &spell.effect.rise {
            add(rise_berserk(&spell.id), &rise.berserk_message);
        }
    }
    for entry in content.miscasts() {
        add(miscast_message(&entry.id), &entry.message);
    }
    for mishap in content.mishaps() {
        add(mishap_message(&mishap.id), &mishap.message);
    }
    for row in &content.ruin().rows {
        add(ruin_says(content, row, false), &row.one);
        add(ruin_says(content, row, true), &row.many);
    }
    for (_, def) in content.talents().iter() {
        for (_, message) in reprieves(def) {
            add(reprieve(&def.id), message);
        }
    }
}

// --- An item's description's words from the data (#2035) -------------------------------------

/// Slot `id`'s `mark`, what the inventory says of a thing in it ("being worn").
pub fn slot_mark(id: &str) -> String {
    format!("slot:{id}.mark")
}

/// Unique `id`'s power in words (its `phrase`), a line of its description.
pub fn unique_phrase(id: &str) -> String {
    format!("unique:{id}.phrase")
}

/// Unique `id`'s `lore`, a line of its description.
pub fn unique_lore(id: &str) -> String {
    format!("unique:{id}.lore")
}

/// Spell `id`'s `description`, what a grimoire or a wand says it does.
pub fn spell_description(id: &str) -> String {
    format!("spell:{id}.description")
}

/// Rod effect `id`'s `name` (`Holds Digging`).
pub fn rod_effect_name(id: &str) -> String {
    format!("rod_effect:{id}.name")
}

/// Rod effect `id`'s `description`.
pub fn rod_effect_description(id: &str) -> String {
    format!("rod_effect:{id}.description")
}

/// The words an item's description takes from the data: a slot's mark, a unique's power and
/// lore, a spell's and a rod effect's description, and a rod effect's name.
fn describe_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for slot in content.slots() {
        add(slot_mark(&slot.id), &slot.mark);
    }
    for def in content.uniques() {
        if let Some(phrase) = &def.phrase {
            add(unique_phrase(&def.id), phrase);
        }
        if let Some(lore) = &def.lore {
            add(unique_lore(&def.id), lore);
        }
    }
    for spell in content.spells() {
        add(spell_description(&spell.id), &spell.description);
    }
    for effect in content.rod_effects() {
        add(rod_effect_name(&effect.id), &effect.name);
        add(rod_effect_description(&effect.id), &effect.description);
    }
}

// --- What screens and the look line show of the data (#2035) -----------------------------------

/// The data's words the screens and the look line show as written: a race's, a class's, a
/// faith's, a talent's, a skill's, a scenario's, and a birth sign animal's and element's `description`, a
/// lucky day's, a terrain's and a chest band's `description` (the look
/// line and a feature underfoot), an animated body's `moves`, a status's `death` (the death
/// and graveyard screens: "Bled to death"), and a resource's `short` (the status bar).
fn shown_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.resources() {
        add(resource_short(&def.id), &def.short);
    }
    for def in &content.races().list {
        add(description("race", &def.id), &def.description);
    }
    for def in content.classes() {
        add(description("class", &def.id), &def.description);
    }
    for (_, def) in content.faiths().list() {
        add(description("faith", &def.id), &def.description);
    }
    for (_, def) in content.talents().iter() {
        add(description("talent", &def.id), &def.description);
    }
    for def in content.skills() {
        add(description("skill", &def.id), &def.description);
    }
    for def in content.terrain_defs() {
        if let Some(text) = &def.description {
            add(description("terrain", &def.id), text);
        }
    }
    for band in &content.chests().bands {
        add(description("chest", &band.id), &band.description);
    }
    for body in &content.animated().bodies {
        if let Some(moves) = &body.spec.moves {
            add(animated_moves(&body.spec.id), moves);
        }
    }
    for def in content.status_defs() {
        if let Some(death) = &def.death {
            add(status_death(&def.id), death);
        }
    }
    for def in content.scenarios.values() {
        add(description("scenario", &def.id), &def.description);
    }
    if let Some(horoscope) = content.horoscope() {
        for part in &horoscope.animals {
            add(description("sign_animal", &part.id), &part.description);
        }
        for part in &horoscope.elements {
            add(description("sign_element", &part.id), &part.description);
        }
        for day in &horoscope.lucky {
            add(lucky_description(day), &day.description);
        }
    }
}

/// A lucky day's legend, its `description`: `lucky:<element>.<animal>.<day>.description`.
pub fn lucky_description(day: &super::Lucky) -> String {
    format!(
        "lucky:{}.{}.{}.description",
        day.element, day.animal, day.day
    )
}

/// A `description` the screens show: `<kind>:<id>.description` (`race`, `class`, `faith`, `talent`,
/// `skill`, `terrain`, `chest`, `scenario`, `sign_animal`, `sign_element`: the kinds of their
/// lexicon ids).
pub fn description(kind: &str, id: &str) -> String {
    format!("{kind}:{id}.description")
}

/// How an animated body moves, its `moves`: `animated:<body>.moves` ("rocking").
pub fn animated_moves(body: &str) -> String {
    format!("animated:{body}.moves")
}

/// How a status kills, its `death`: `status:<id>.death` ("Bled to death").
pub fn status_death(id: &str) -> String {
    format!("status:{id}.death")
}

// --- The classes' lines (L4c, #1914) -----------------------------------------------------------

/// The `news` of talent `def`'s `weapon_ranks` and `take_up_skills` effects (said when a class
/// table gives it), each key with its text: `talent:<id>.effects.<kind>.news`.
pub fn handed_news(def: &TalentDef) -> impl Iterator<Item = (String, &str)> {
    use crate::talent::GrowthEffect::{TakeUpSkills, WeaponRanks};
    (def.effects.iter()).filter_map(|effect| {
        let (kind, news) = match effect {
            TalentEffect::Growth(WeaponRanks { news, .. }) => ("weapon_ranks", news),
            TalentEffect::Growth(TakeUpSkills { news, .. }) => ("take_up_skills", news),
            _ => return None,
        };
        let text = news.as_deref()?;
        Some((format!("talent:{}.effects.{kind}.news", def.id), text))
    })
}

/// Resource `id`'s `lacking` words (`resources.toml`'s `[[resource]]`), said in
/// `classes.lacking`.
pub fn lacking(id: &str) -> String {
    format!("resources:resource.{id}.lacking")
}

/// The prayer's words for `face` (`prayer.toml`'s `[words]`), `field` one of `pray`, `fizzle`,
/// and `nothing`: `prayer:words.<face>.<field>` (L4d, #1915).
pub fn prayer_words(face: super::Face, field: &str) -> String {
    format!("prayer:words.{}.{field}", face.id())
}

/// Use `id`'s words for `face` (`prayer.toml`'s `[[use]]`): `prayer:use.<id>.<face>`.
pub fn prayer_use(id: super::PrayerUse, face: super::Face) -> String {
    format!("prayer:use.{}.{}", id.id(), face.id())
}

/// The classes' lines: the talents' handed-out news, the resources' `lacking`, and the prayer's
/// words.
fn class_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for (_, def) in content.talents().iter() {
        for (key, text) in handed_news(def) {
            add(key, text);
        }
    }
    for def in content.resources() {
        if let Some(words) = &def.lacking {
            add(lacking(&def.id), words);
        }
    }
    if let Some(rules) = content.prayer() {
        for face in [super::Face::Holy, super::Face::Dark] {
            let words = rules.words(face);
            add(prayer_words(face, "pray"), &words.pray);
            add(prayer_words(face, "fizzle"), &words.fizzle);
            add(prayer_words(face, "nothing"), &words.nothing);
        }
        for def in &rules.uses {
            for face in [super::Face::Holy, super::Face::Dark] {
                add(prayer_use(def.id, face), def.words.of(face));
            }
        }
    }
}

// --- The turn's lines (L4o, #1926): what the player perceives as a turn ends -------------------

/// Trap `id`'s `message`, said as the player perceives it go off (`traps.toml`'s `[[trap]]`).
pub fn trap_message(id: &str) -> String {
    format!("trap:{id}.message")
}

/// Disease `id`'s `onset`, said as it shows.
pub fn disease_onset(id: &str) -> String {
    format!("disease:{id}.onset")
}

/// Disease `id`'s symptom `i` (from 0), its `symptoms[i]`.
fn disease_symptom_at(id: &str, i: usize) -> String {
    format!("disease:{id}.symptoms.{i}")
}

/// Disease `def`'s symptom at its progression number `n` (1 for the first) when the check
/// fails ([`super::DiseaseDef::symptom_index`]); `None` when it has none.
pub fn disease_symptom(def: &super::DiseaseDef, n: u16) -> Option<String> {
    Some(disease_symptom_at(&def.id, def.symptom_index(n)?))
}

/// Resource `id`'s `ended`, said as it ends (`resources.toml`'s `[[resource]]`).
pub fn resource_ended(id: &str) -> String {
    format!("resources:resource.{id}.ended")
}

/// Resource `id`'s `short`, its status bar label ("Rage", "Sta"; #2062).
pub fn resource_short(id: &str) -> String {
    format!("resources:resource.{id}.short")
}

/// The turn's lines: the traps' messages, the diseases' onsets and symptoms, and the resources'
/// `ended`.
fn turn_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.trap_defs() {
        if let Some(message) = &def.message {
            add(trap_message(&def.id), message);
        }
    }
    for def in content.disease_defs() {
        add(disease_onset(&def.id), &def.onset);
        for (i, symptom) in def.symptoms.iter().enumerate() {
            add(disease_symptom_at(&def.id, i), symptom);
        }
    }
    for def in content.resources() {
        if let Some(ended) = &def.ended {
            add(resource_ended(&def.id), ended);
        }
    }
}

/// Adds every line above to English by its key; a line naming a role is refused (none is given
/// one).
pub(super) fn add_all(content: &mut Content) -> Result<(), ContentError> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut add = |key: String, text: &str| lines.push((key, text.to_string()));
    for def in content.status_defs() {
        if def.stages.is_empty()
            && let Some(start) = &def.start
        {
            add(status_start(def), start);
        }
        for (i, stage) in def.stages.iter().enumerate() {
            add(status_stage(def, i + 1), stage);
        }
        if let Some(end) = &def.end {
            add(status_end(def), end);
        }
        if let Some(treat) = &def.treat {
            add(status_treat(def, true), &treat.made);
            add(status_treat(def, false), &treat.failed);
        }
    }
    for def in content.disease_defs() {
        add(disease_recovered(&def.id), &def.recovered);
    }
    for def in content.creature_defs() {
        if let Some(message) = def.on_eat.as_ref().and_then(|o| o.message.as_ref()) {
            add(on_eat(&def.id), message);
        }
        if let Some(words) = &def.depart {
            add(depart(&def.id), words);
        }
    }
    for def in content.uniques() {
        if let Some(super::Power::Ward { message, .. } | super::Power::Negate { message, .. }) =
            &def.power
        {
            add(ward(&def.id), message);
        }
    }
    for def in content.object_defs() {
        if let Some(words) = def.flask.as_ref().and_then(|f| f.words.as_ref()) {
            add(flask_words(&def.id), words);
        }
    }
    for spell in content.spells() {
        if let Some(c) = &spell.effect.charm {
            add(charm(&spell.id, false), &c.message);
            add(charm(&spell.id, true), &c.ends);
        }
    }
    if let Some(rules) = content.familiars() {
        for (creature, line) in &rules.arrives {
            add(familiar_arrives(&content.creature(*creature).id), line);
        }
    }
    if let Some(corpses) = content.hunger().and_then(|h| h.corpses.as_ref()) {
        for (skill, news) in &corpses.skills {
            add(skill_gift(skill, true), &news.learn);
            add(skill_gift(skill, false), &news.rise);
        }
    }
    character_lines(content, &mut add);
    magic_lines(content, &mut add);
    crafting_lines(content, &mut add);
    class_lines(content, &mut add);
    fixture_lines(content, &mut add);
    creature_lines(content, &mut add);
    turn_lines(content, &mut add);
    for (id, level) in &content.levels {
        if let Some(line) = &level.arrival {
            add(level_arrival(id), line);
        }
    }
    for def in content.dungeons.values() {
        if let Some(line) = &def.arrival {
            add(dungeon_arrival(&def.id), line);
        }
    }
    for def in content.scenarios.values() {
        if let Some(line) = &def.arrival {
            add(scenario_arrival(&def.id), line);
        }
    }
    describe_lines(content, &mut add);
    shown_lines(content, &mut add);
    people_lines(content, &mut add);
    place_notes(content, &mut add);
    super::data_word::lines(content, &mut add);
    for (key, text) in lines {
        let line = Line::parse(&text);
        if let Some(role) = line.roles().first() {
            return Err(ContentError::new(
                "data",
                format!("`{key}`: \"{text}\" names `{role}`, which this line is not given"),
            ));
        }
        content.english.add_data_line(key, Template::Line(line));
    }
    quest_speech(content)?;
    saved_names(content)?;
    Ok(())
}

/// The character's lines: alignment, the craving, the laws, the oaths, profane gifts, and cold
/// blood.
fn character_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    if let Some(rules) = content.alignment() {
        for def in rules.bands() {
            for (rise, text) in [(true, &def.rise), (false, &def.fall)] {
                if let Some(text) = text {
                    add(alignment_band(def.band, rise), text);
                }
            }
        }
    }
    if let Some(rules) = content.craving() {
        add(craving("first"), &rules.first);
        if let Some(again) = &rules.again {
            add(craving("again.message"), &again.message);
        }
        if let Some(surrender) = &rules.surrender {
            add(craving("surrender.first"), &surrender.first);
        }
        for row in &rules.thresholds {
            add(craving_threshold(content, row.talent), &row.message);
        }
    }
    if let Some(laws) = content.laws() {
        add(law_first("eat"), &laws.eat.first);
        add(law_first("summon"), &laws.summon.first);
    }
    if let Some(oaths) = content.oaths() {
        for band in &oaths.bands {
            add(oath_band(content, band.talent), &band.message);
        }
    }
    for gift in content.profane().iter() {
        let fields = [
            ("line", Some(&gift.line)),
            ("undone", gift.undone.as_ref()),
            ("instead", gift.instead.as_ref()),
        ];
        for (field, text) in fields {
            if let Some(text) = text {
                add(profane(&gift.id, field), text);
            }
        }
    }
    for (_, def) in content.talents().iter() {
        for bands in warmth_bands(def) {
            for (n, band) in bands.iter().enumerate() {
                for (slower, text) in [(true, &band.slower), (false, &band.faster)] {
                    if let Some(text) = text {
                        add(warmth(&def.id, n, slower), text);
                    }
                }
            }
        }
    }
}

/// Crafting's lines: the professions' `failed` and `fumbled`, the rigs' `made` and `snaps`.
fn crafting_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in &content.recipes().list {
        if let Some(verb) = &def.verb {
            add(recipe_verb(&def.id), verb);
        }
    }
    let stock = content.stock();
    let groups = stock.groups.iter().filter_map(|g| g.profession.as_ref());
    for profession in groups.chain(&stock.gems) {
        add(profession.verb_key.clone(), &profession.verb);
    }
    for text in &content.recipes().professions {
        let skill = &content.skill(text.skill).id;
        if let Some(verb) = &text.verb {
            add(profession_verb(skill), verb);
        }
        for (fumbled, line) in [(false, &text.failed), (true, &text.fumbled)] {
            if let Some(line) = line {
                add(profession(skill, fumbled), line);
            }
        }
    }
    for (_, def) in content.rigs().into_iter().flat_map(|rules| rules.iter()) {
        add(rig(&def.id, false), &def.made);
        add(rig(&def.id, true), &def.snaps);
    }
}

// ----- People (`world/people/`, #1924): what quests' people say, a town's goodwill, and a
// hidden location opening. -----

/// Quest `quest`'s line at `path` in its `[[quest]]`: `offer`, `variant.<id>.offer`,
/// `joins`, `binding.words.0`, `taker.<npc>.done`, `side.<npc>.join`, `parting.betray`
/// (a taker and a side by their npc).
pub fn quest(quest: &str, path: &str) -> String {
    format!("quest:{quest}.{path}")
}

/// The words of a quest its giver says that a drawn variant may say instead, each a field of
/// the `[[quest]]` and of its `[[quest.variant]]`s.
pub const QUEST_TEXTS: [&str; 5] = ["offer", "accept", "reminder", "done", "where"];

/// What a reward says of the skill `skill` it teaches: `taught` (true) to one untrained, else
/// `more`. `reward` is the reward's path in the quest: `reward`, `taker.<npc>.reward`, or
/// `on_accept`.
pub fn quest_skill(quest: &str, reward: &str, skill: &str, taught: bool) -> String {
    let field = if taught { "taught" } else { "more" };
    self::quest(quest, &format!("{reward}.skills.{skill}.{field}"))
}

/// Town `id`'s `told` line, said as its goodwill rises.
pub fn town_told(id: &str) -> String {
    format!("town:{id}.told")
}

/// Hidden location `id`'s `opens` line, the location on level `level`.
pub fn hidden_opens(level: &str, id: &str) -> String {
    format!("level:{level}.hidden.{id}.opens")
}

/// The narrator's lines of `world/people/`: a lure quest's binding, a reward's skills, a town's
/// `told`, and a hidden location's `opens`.
fn people_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.quests() {
        let id = &def.id;
        if let Some((_, _, _, binding)) = def.lure() {
            for (n, words) in binding.words.iter().enumerate() {
                add(quest(id, &format!("binding.words.{n}")), words);
            }
            if let Some(line) = &binding.shatters {
                add(quest(id, "binding.shatters"), line);
            }
            if let Some(line) = &binding.reforms {
                add(quest(id, "binding.reforms"), line);
            }
        }
        let takers =
            (def.quarrel.takers.iter()).map(|t| (format!("taker.{}.reward", t.npc), &t.reward));
        let accept = (def.on_accept.iter()).map(|o| ("on_accept".to_string(), &o.reward));
        let rewards = std::iter::once(("reward".to_string(), &def.reward)).chain(takers);
        for (path, reward) in rewards.chain(accept) {
            for taught in &reward.skills {
                let skill = &content.skill(taught.skill).id;
                for (yes, text) in [(true, &taught.taught), (false, &taught.more)] {
                    if let Some(text) = text {
                        add(quest_skill(id, &path, skill, yes), text);
                    }
                }
            }
        }
    }
    for town in content.towns() {
        add(town_told(&town.id), &town.told);
    }
    for level in content.levels.values() {
        for hidden in level.hidden.values() {
            if let Some(line) = &hidden.opens {
                add(hidden_opens(&level.id, &hidden.id), line);
            }
        }
    }
}

// The fixtures' and the items' lines (L4h, #1919): outcomes, special rooms, and a barred slot.

/// Line `n` of `outcome` (`data/outcomes/<table>.toml`'s `[[outcome]]` of its id): its
/// `message` when it has one line, else its `messages[n]`.
pub fn outcome_message(outcome: &OutcomeDef, n: usize) -> String {
    let (table, id) = (&outcome.table, &outcome.id);
    match outcome.messages.len() {
        1 => format!("outcome:{table}.outcome.{id}.message"),
        _ => format!("outcome:{table}.outcome.{id}.messages.{n}"),
    }
}

/// `outcome`'s `otherwise` line.
pub fn outcome_otherwise(outcome: &OutcomeDef) -> String {
    format!("outcome:{}.outcome.{}.otherwise", outcome.table, outcome.id)
}

/// Room effect `def`'s line `field` (`enter`, `near`, `pervade`, `arrive`; a shrine's
/// `shrine.welcome`, `shrine.unwelcome`).
pub fn room_effect(def: &RoomEffectDef, field: &str) -> String {
    format!("room_effect:{}.{field}", def.id)
}

/// What talent `talent` says when something is put on in slot `slot` it bars (the `refused`
/// of its `bars_slot` effect for the slot).
pub fn barred(talent: &str, slot: &str) -> String {
    format!("talent:{talent}.effects.bars_slot.{slot}.refused")
}

/// The fixtures' and the items' lines: every outcome's, every room effect's, and every barred
/// slot's.
fn fixture_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for table in content.outcomes().iter() {
        for outcome in &table.outcomes {
            for (n, line) in outcome.messages.iter().enumerate() {
                add(outcome_message(outcome, n), line);
            }
            if let Some(line) = &outcome.otherwise {
                add(outcome_otherwise(outcome), line);
            }
        }
    }
    for def in content.room_effects().defs() {
        add(room_effect(def, "enter"), &def.enter);
        let fields = [
            ("near", &def.near),
            ("pervade", &def.pervade),
            ("arrive", &def.arrive),
        ];
        for (field, line) in fields {
            if let Some(line) = line {
                add(room_effect(def, field), line);
            }
        }
        if let Some(shrine) = &def.shrine {
            add(room_effect(def, "shrine.welcome"), &shrine.welcome);
            add(room_effect(def, "shrine.unwelcome"), &shrine.unwelcome);
        }
    }
    for (_, def) in content.talents().iter() {
        let whiles = def.effects_while.iter().map(|(_, e)| e);
        for effect in def.effects.iter().chain(whiles) {
            if let TalentEffect::Slots(SlotEffect::BarsSlot { slot, refused, .. }) = effect {
                add(barred(&def.id, slot), refused);
            }
        }
    }
}

// The creatures' lines (`world/creatures/`, #1916).

/// Creature type `id`'s `first_sight` line, said the first time a run the player sees one.
pub fn first_sight(id: &str) -> String {
    format!("creature:{id}.first_sight")
}

/// The creatures' lines: a type's first sight.
fn creature_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.creature_defs() {
        if let Some(line) = &def.first_sight {
            add(first_sight(&def.id), line);
        }
    }
}

// ----- Names kept in saves and the quest log (#1999): a shop's sign, an encounter's party, a
// month, and a burial ground's epitaphs, said by their keys so a saved line or a level's graves
// render in any language. -----

/// Shop kind `id`'s sign (its `title`), which names its `keeper` (`shops.toml`).
pub fn shop_sign(id: &str) -> String {
    format!("shop:{id}.title")
}

/// The name of encounter entry `entry` of campaign `campaign`'s tables (an entry's id is its
/// campaign's), as the data writes it, article and all: "a band of footpads".
pub fn encounter_name(campaign: &str, entry: &str) -> String {
    format!("encounter:{campaign}.{entry}.name")
}

/// Month `month`'s name (1 to 12; `calendar.toml`'s `months`, numbered from 0).
pub fn month(month: u32) -> String {
    format!("calendar:months.{}", month.clamp(1, 12) - 1)
}

/// The words of level `level`'s fixed grave of legend character `glyph` (its `epitaph`).
pub fn grave(level: &str, glyph: char) -> String {
    format!("level:{level}.legend.{glyph}.epitaph")
}

/// Level `level`'s words for one of the run's dead laid there (`[burial]`'s `epitaph`), which
/// name them (`name`) and how they died (`death`).
pub fn burial_epitaph(level: &str) -> String {
    format!("level:{level}.burial.epitaph")
}

// ----- What is said of places (#1998): a legend's named place's note, said by its key so the
// look line renders it in any language. The names of places are lexicon words
// (`super::place_word`, #2062). -----

/// Scenario `id`'s `arrival` line, said after the welcome.
pub fn scenario_arrival(id: &str) -> String {
    format!("scenario:{id}.arrival")
}

/// The notes of places, which name no role: every legend's named place's note
/// ([`super::Place::key`]).
fn place_notes(content: &Content, add: &mut impl FnMut(String, &str)) {
    let templates = (content.levels.values()).chain(content.vaults.iter().map(|v| &v.template));
    for place in templates.flat_map(|t| t.places.values()) {
        if let Some(note) = &place.note {
            add(place.note_key(), note);
        }
    }
}

/// The roles a background story's line is given (`story:<scope>.<line>`,
/// `crate::background::lines`): `who`, the character (a phrase of their name and gender), and the
/// slot words, `guardian`, `home`, `trade` (`story_word:` lexicon ids).
pub const STORY_ROLES: [&str; 4] = ["who", "guardian", "home", "trade"];

/// Adds the names above and the background story's lines to English by their keys, each naming
/// only the roles it is given.
fn saved_names(content: &mut Content) -> Result<(), ContentError> {
    let mut lines: Vec<(String, String, &[&str])> = Vec::new();
    for def in content.shops().map_or(&[][..], |s| s.defs()) {
        lines.push((shop_sign(&def.id), def.title.clone(), &["keeper"]));
    }
    for table in &content.encounter_tables {
        for entry in &table.entries {
            let key = encounter_name(&table.campaign, &entry.id);
            lines.push((key, entry.name.clone(), &[]));
        }
    }
    for n in 1..=12 {
        lines.push((month(n), content.calendar().month(n).to_string(), &[]));
    }
    // A background story's lines (#2062): the character and the slot words.
    if let Some(background) = content.background() {
        for (key, text) in background.lines() {
            lines.push((key, text, &STORY_ROLES));
        }
    }
    for (id, level) in &content.levels {
        let Some(burial) = &level.burial else {
            continue;
        };
        if let Some(epitaph) = &burial.epitaph {
            lines.push((burial_epitaph(id), epitaph.clone(), &super::EPITAPH_ROLES));
        }
        for (glyph, words) in burial.epitaphs.values() {
            lines.push((grave(id, *glyph), words.clone(), &[]));
        }
    }
    for (key, text, roles) in lines {
        let line = Line::parse(&text);
        if let Some(role) = line.roles().into_iter().find(|r| !roles.contains(r)) {
            return Err(ContentError::new(
                "data",
                format!("`{key}`: \"{text}\" names `{role}`, which this line is not given"),
            ));
        }
        content.english.add_data_line(key, Template::Line(line));
    }
    Ok(())
}

/// Adds what quests' people say to English by its key: a giver's words (and a drawn variant's),
/// an escort's, a taker's, a side's, and the parting's betrayal; and the quest's `name`. Said
/// by a person, a line takes the base form only; a rolled quest's words may name what the roll drew
/// (`content/quest_roll.rs`'s roles), any other quest's none.
fn quest_speech(content: &mut Content) -> Result<(), ContentError> {
    let mut lines: Vec<(String, String, &[&str])> = Vec::new();
    for def in content.quests() {
        let id = &def.id;
        let roles: &'static [&'static str] = if def.is_rolled() {
            &super::quest_roll::ROLES
        } else {
            &[]
        };
        let mut add = |path: String, text: &str, roles: &'static [&'static str]| {
            lines.push((quest(id, &path), text.to_string(), roles));
        };
        // Its name (`QuestDef::name_arg`), which names no role.
        add("name".into(), &def.name, &[]);
        let own = [
            Some(&def.texts.offer),
            def.texts.accept.as_ref(),
            Some(&def.texts.reminder),
            Some(&def.texts.done),
            def.where_.as_ref(),
        ];
        // A text a quest's variants each say instead is empty on the quest.
        for (field, text) in QUEST_TEXTS.iter().zip(own) {
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                add(field.to_string(), text, roles);
            }
        }
        if let Some(super::Rolled::Variants(variants)) = &def.roll {
            for v in variants {
                let theirs = [&v.offer, &v.accept, &v.reminder, &v.done, &v.where_];
                for (field, text) in QUEST_TEXTS.iter().zip(theirs) {
                    if let Some(text) = text {
                        add(format!("variant.{}.{field}", v.id), text, roles);
                    }
                }
            }
        }
        if let Some((_, early)) = &def.min_level {
            add("early".into(), early, &[]);
        }
        if let Some(says) = def.on_accept.as_ref().and_then(|o| o.says.as_ref()) {
            add("on_accept.says".into(), says, &[]);
        }
        if let super::Goal::Escort { joins, thanks, .. } = &def.goal {
            add("joins".into(), joins, &[]);
            if let Some(thanks) = thanks {
                add("thanks".into(), thanks, &[]);
            }
        }
        for t in &def.quarrel.takers {
            add(format!("taker.{}.done", t.npc), &t.done, &[]);
            if let Some(betrayed) = &t.betrayed {
                add(format!("taker.{}.betrayed", t.npc), betrayed, &[]);
            }
        }
        for s in &def.quarrel.sides {
            add(format!("side.{}.offer", s.npc), &s.offer, &[]);
            add(format!("side.{}.join", s.npc), &s.join, &[]);
            add(format!("side.{}.refuse", s.npc), &s.refuse, &[]);
        }
        if let Some(parting) = &def.quarrel.parting {
            add("parting.betray".into(), &parting.betray, &[]);
        }
    }
    for (key, text, roles) in lines {
        let line = Line::parse(&text);
        if let Some(role) = line.roles().into_iter().find(|r| !roles.contains(r)) {
            return Err(ContentError::new(
                "data",
                format!("`{key}`: \"{text}\" names `{role}`, which this line is not given"),
            ));
        }
        content.english.add_data_line(key, Template::Line(line));
    }
    Ok(())
}

#[cfg(test)]
#[path = "data_line_tests.rs"]
mod tests;
