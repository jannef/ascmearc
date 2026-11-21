//! Crafting by recipe (professions; docs/design/professions.md §6, §8;
//! docs/dev/crafting.md): what a profession's screen lists, and [`Action::Craft`].
//!
//! - **The rows** ([`recipe_rows`], what [`PerceptionView::recipes`] reads): a profession's
//!   recipes in path order, for one trained in the skill (none for the untrained: every recipe
//!   is a skill's, #1387), each with what it takes in words the player sees elsewhere ("an
//!   anvil, about 10 minutes"; never a difficulty or odds) and the first reason it cannot be made
//!   now, if any ([`RecipeLack`]): computed here, from the pack, the cells beside the player, and
//!   the skills, so presentation never decides availability.
//! - **Known inputs** ([`known_object`]): an object input counts only once the player knows its
//!   type ([`Lore::is_identified`]: a labelled potion, flask, scroll, wand, or rod does not), in
//!   the rows, the wizard's steps, and the action alike, so no row tells what an unknown stack is
//!   (#1383).
//! - **The action** ([`Action::Craft`], `craft:ID[:POS,...]`): refused, taking no time, for one
//!   untrained in the recipe's skill ("You are not trained in Woodcraft."; `World::untrained`,
//!   which `temper:` and `whittle:` ask too); a built-in recipe's row runs its own action
//!   (`temper:`, `whittle:`), so `craft:` refuses it, taking no time, but for Cut and Set, which are
//!   `craft:cut:LETTER` and `craft:set:ITEM,GEM` themselves (`world/crafting/cut.rs`, `world/crafting/set.rs`); a
//!   generic recipe is made by `world/crafting/craft_make.rs` (the forge's weapons, the hand-made ones,
//!   the arrows, the brews).
//!
//! [`Action::Craft`]: crate::action::Action::Craft
//! [`PerceptionView::recipes`]: crate::perception::PerceptionView::recipes

use crate::content::{
    Content, InputKind, MaterialId, ObjectId, PartSource, RaceDef, RecipeDef, RecipeId,
    RecipeInput, RecipeUse, TerrainId, counted,
};
use crate::geom::{Dir, Pos};
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::scroll_label::Lore;
use crate::skills::SkillId;
use crate::text::article;
use crate::time::Duration;
use crate::world::{Actor, LevelState, Spent, World};

/// One recipe as the profession's screen lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeRow {
    pub id: RecipeId,
    pub name: String,
    /// What it takes, in words: "a metal weapon or armour, an anvil, about 10 minutes".
    pub takes: String,
    /// Why it cannot be made now, the first reason that applies; `None` when it can.
    pub unavailable: Option<RecipeLack>,
}

/// Why a recipe cannot be made now (the screen greys its row with [`RecipeLack::reason`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeLack {
    /// The player's trained value is below the recipe's `from`.
    Skill,
    /// No station of this terrain beside the player.
    Station(TerrainId),
    /// No knife carried.
    Knife,
    /// No launcher ready that shoots this result (a bow for arrows).
    Launcher(ObjectId),
    /// No stack of stock fits a chosen part, or holds a stock input: this material's group is
    /// what is missing.
    Stock(MaterialId),
    /// Too few of an input carried.
    Input(ObjectId),
    /// No cut loose gem carried for a part that must be one (a gold amulet's setting, Set's).
    Gem,
    /// Nothing carried with an unset ornament part (Set's).
    Setting,
    /// Nothing carried an enchanting recipe may work (`world/crafting/enchant.rs`).
    Nothing,
    /// Only items with no room for another affix (Quicken's).
    Room,
    /// No crystal carried, for an enchanting recipe that takes one.
    Crystal,
}

impl RecipeLack {
    /// The reason, as the row says it, a clause of `crafting.lacking_*`: "no anvil beside you".
    pub fn reason(self, content: &Content) -> Msg {
        match self {
            RecipeLack::Skill => msg::crafting::LACKING_SKILL.into(),
            RecipeLack::Station(t) => {
                Msg::new(msg::crafting::LACKING_STATION).role("station", terrain(content, t))
            }
            RecipeLack::Knife => msg::crafting::LACKING_KNIFE.into(),
            RecipeLack::Launcher(o) => Msg::new(msg::crafting::LACKING_LAUNCHER).role(
                "launcher",
                super::fletch::launcher_word(content, o).to_string(),
            ),
            RecipeLack::Stock(m) => {
                let def = content.material(m);
                let stock = def.group.as_deref().unwrap_or(&def.name).to_string();
                Msg::new(msg::crafting::LACKING_STOCK).role("stock", stock)
            }
            RecipeLack::Input(o) => Msg::new(msg::crafting::LACKING_INPUT).role(
                "what",
                Phrase::id(format!("object:{}", content.object(o).id)),
            ),
            RecipeLack::Gem => msg::crafting::LACKING_GEM.into(),
            RecipeLack::Setting => msg::crafting::LACKING_SETTING.into(),
            RecipeLack::Nothing => msg::crafting::LACKING_ENCHANTABLE.into(),
            RecipeLack::Room => msg::crafting::LACKING_ROOM.into(),
            RecipeLack::Crystal => msg::crafting::LACKING_CRYSTAL.into(),
        }
    }

    /// Its id in dumps: `skill`, `station:anvil`, `knife`, `launcher`, `stock:metal`,
    /// `input:arrowhead`, `gem`, `setting`, `nothing`, `room`, `crystal`.
    pub fn id(self, content: &Content) -> String {
        match self {
            RecipeLack::Skill => "skill".into(),
            RecipeLack::Station(t) => format!("station:{}", content.terrain(t).id),
            RecipeLack::Knife => "knife".into(),
            RecipeLack::Launcher(_) => "launcher".into(),
            RecipeLack::Stock(m) => {
                let def = content.material(m);
                format!("stock:{}", def.group.as_deref().unwrap_or(&def.id))
            }
            RecipeLack::Input(o) => format!("input:{}", content.object(o).id),
            RecipeLack::Gem => "gem".into(),
            RecipeLack::Setting => "setting".into(),
            RecipeLack::Nothing => "nothing".into(),
            RecipeLack::Room => "room".into(),
            RecipeLack::Crystal => "crystal".into(),
        }
    }
}

/// The rows of profession `skill` for `player` on `level` (the module docs).
pub(crate) fn recipe_rows(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    race: Option<&RaceDef>,
    skill: SkillId,
) -> Vec<RecipeRow> {
    if player.skills.get(skill).is_none() {
        return Vec::new();
    }
    (content.recipes().of(skill))
        .filter(|(_, recipe)| of_race(recipe, race))
        .map(|(id, recipe)| RecipeRow {
            id,
            name: recipe.name.clone(),
            takes: takes(content, level, player, recipe),
            unavailable: lack(content, lore, level, player, recipe),
        })
        .collect()
}

/// Whether a character of `race` may make `recipe`: any, unless it is one race's own craft (the
/// halfling's pipe, `RecipeDef::race`), which only that race sees or makes.
pub(crate) fn of_race(recipe: &RecipeDef, race: Option<&RaceDef>) -> bool {
    recipe
        .race
        .as_deref()
        .is_none_or(|own| race.is_some_and(|r| r.id == own))
}

/// Terrain `t` as a line names it (a station, a wall): `terrain:<id>`.
pub(super) fn terrain(content: &Content, t: TerrainId) -> Phrase {
    Phrase::id(format!("terrain:{}", content.terrain(t).id))
}

/// The station `station` beside `at` on `level`: the first cell around it (in [`Dir::ALL`]
/// order) whose terrain is the station or works as it ([`Content::works_as`]: a makeshift anvil
/// is an anvil). The one station test every recipe's [`lack`] shares.
pub(crate) fn station_beside(
    content: &Content,
    level: &LevelState,
    at: Pos,
    station: TerrainId,
) -> Option<Pos> {
    (Dir::ALL.iter().map(|&d| at.step(d))).find(|&p| {
        level
            .terrain_at(p)
            .is_some_and(|t| content.works_as(t) == station)
    })
}

/// The first reason `player` cannot make `recipe` now, in the screen's order: skill, station,
/// knife, Set's item and gem, launcher, stock, inputs, a gem part. `"stock"` parts (but a
/// `count = "fletch"` recipe's wood on wooded ground, `world/crafting/fletch.rs`), a required part's
/// sources, object inputs (those a part's sources name only there), stock inputs, and gem parts
/// are looked at, an object input only of a type the player knows. A stock input (a bow's sinew) wants one stack holding all it takes, as the
/// wizard picks one.
pub(super) fn lack(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: &RecipeDef,
) -> Option<RecipeLack> {
    let value = player.skills.value(recipe.skill).unwrap_or(0);
    if value < recipe.from {
        return Some(RecipeLack::Skill);
    }
    if let Some(station) = recipe.station
        && station_beside(content, level, player.pos, station).is_none()
    {
        return Some(RecipeLack::Station(station));
    }
    if recipe.knife && !(player.inventory.iter()).any(|s| content.is_knife(s.object)) {
        return Some(RecipeLack::Knife);
    }
    if recipe.uses == Some(RecipeUse::Set) {
        return super::set::lack(content, player);
    }
    if let Some(RecipeUse::Enchant(_)) = recipe.uses {
        return super::enchant::lack(content, player, recipe);
    }
    let result = recipe.result.as_ref();
    if let Some(result) = result.filter(|_| recipe.launcher)
        && !super::fletch::shoots(player, content, result.object)
    {
        return Some(RecipeLack::Launcher(result.object));
    }
    let carries = |materials: &[MaterialId]| {
        (player.inventory.iter())
            .any(|stack| stack_material(content, stack).is_some_and(|m| materials.contains(&m)))
    };
    let mut sourced = Vec::new();
    for part in &recipe.parts {
        let Some(result) = result else { break };
        match &part.source {
            PartSource::Stock => {
                let worked = worked_materials(content, recipe, result.object, part.part);
                let free = super::fletch::part_free(content, level, player, recipe, part.part);
                if !free && !carries(&worked) {
                    return worked.first().map(|&m| RecipeLack::Stock(m));
                }
            }
            PartSource::Sources {
                input,
                stock,
                optional,
            } => {
                sourced.extend(*input);
                let pack = &player.inventory;
                let held = input.is_some_and(|o| pack.iter().any(|s| known_object(lore, s, o)));
                if !optional && !held && !carries(stock) {
                    return match (input, stock.first()) {
                        (Some(o), _) => Some(RecipeLack::Input(*o)),
                        (None, m) => m.map(|&m| RecipeLack::Stock(m)),
                    };
                }
            }
            PartSource::Gem => {}
        }
    }
    for input in &recipe.inputs {
        let fitting = (player.inventory.iter()).filter(|s| input_fits(lore, input, s));
        let Some(object) = input.object() else {
            if !fitting.into_iter().any(|s| s.count >= input.count) {
                return input_material(content, input).map(RecipeLack::Stock);
            }
            continue;
        };
        if sourced.contains(&object) {
            continue;
        }
        if fitting.map(|s| s.count).sum::<u32>() < input.count {
            return Some(RecipeLack::Input(object));
        }
    }
    let gem_part = (recipe.parts.iter()).any(|p| p.source == PartSource::Gem);
    let gem = (player.inventory.iter()).any(|s| super::craft_make::cut_gem(content, s).is_some());
    (gem_part && !gem).then_some(RecipeLack::Gem)
}

/// Whether `stack` is what `input` takes: a stack of its object, of a type the player knows
/// ([`known_object`]), or a piece of its material or of a material of its group (a crossbow's
/// lock: an ingot of any metal).
pub(super) fn input_fits(
    lore: Lore<'_>,
    input: &RecipeInput,
    stack: &crate::object::Stack,
) -> bool {
    let content = lore.content;
    match &input.kind {
        InputKind::Object(object) => known_object(lore, stack, *object),
        InputKind::Material(material) => stack_material(content, stack) == Some(*material),
        InputKind::Group(group) => stack_material(content, stack)
            .is_some_and(|m| content.material(m).group.as_deref() == Some(group.as_str())),
    }
}

/// Whether `stack` is of `object` as the player knows it: of that object, and of a type they
/// have identified (module docs). An unknown flask of oil is no flask of oil to a recipe, so its
/// row cannot tell what it is (#1383). A unique is never an input: no craft spends one (#1975).
/// The one test every object input is matched by.
pub(super) fn known_object(lore: Lore<'_>, stack: &crate::object::Stack, object: ObjectId) -> bool {
    stack.object == object && lore.is_identified(object) && !stack.is_unique()
}

/// The material a stock input names, or the first of its group: what its lack and its words
/// are said by.
fn input_material(content: &Content, input: &RecipeInput) -> Option<MaterialId> {
    match &input.kind {
        InputKind::Object(_) => None,
        InputKind::Material(material) => Some(*material),
        InputKind::Group(group) => (content.materials().iter())
            .position(|m| m.group.as_deref() == Some(group.as_str()))
            .and_then(|i| u16::try_from(i).ok())
            .map(MaterialId),
    }
}

/// A stock input in words: "a length of sinew", "an ingot" (of any metal: the form word alone,
/// [`crate::piece::any_names`]).
fn stock_input_words(content: &Content, input: &RecipeInput) -> Option<String> {
    let material = input_material(content, input)?;
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let (one, many) = match &input.kind {
        InputKind::Group(group) => {
            let of_group: Vec<MaterialId> = (0..content.materials().len())
                .filter_map(|i| u16::try_from(i).ok().map(MaterialId))
                .filter(|&m| content.material(m).group.as_deref() == Some(group.as_str()))
                .collect();
            crate::piece::any_names(content, piece, &of_group)?
        }
        _ => crate::piece::names(content, piece, material),
    };
    Some(counted(&one, &many, input.count))
}

/// Whether a part's source is stock alone, the steps built so far: `"stock"`, or a table of
/// `stock` with no input that cannot be left off (a spear's haft, `{ stock = ["wood"] }`, any
/// profession's; docs/design/recipes.md §10).
pub(super) fn stock_only(source: &PartSource) -> bool {
    matches!(
        source,
        PartSource::Stock
            | PartSource::Sources {
                input: None,
                optional: false,
                ..
            }
    )
}

/// The materials part `part` of `object` may be made of as `recipe` takes it, in the part's list
/// order: what a stock stack must be of. A `"stock"` part takes what the recipe's profession
/// works; a table's `stock` list (any profession's) what it names.
pub(super) fn worked_materials(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
) -> Vec<MaterialId> {
    let source = (recipe.parts.iter()).find(|p| p.part == part);
    if let Some(PartSource::Sources { stock, .. }) = source.map(|p| &p.source) {
        return stock.clone();
    }
    (content.object(object).made_of(part).iter().copied())
        .filter(|&m| {
            (content.stock().profession_of(content, m)).is_some_and(|p| p.skill == recipe.skill)
        })
        .collect()
}

/// The material of `stack` when it is stock (a piece): its main part's.
pub(super) fn stack_material(
    content: &Content,
    stack: &crate::object::Stack,
) -> Option<MaterialId> {
    if Some(stack.object) != content.object_id(crate::piece::ID) {
        return None;
    }
    stack.parts(content).first().map(|p| p.material)
}

/// What `recipe` takes, in words, joined with commas: what a built-in one works on, the parts'
/// stock and sources (a stock part priced for the batch at its most; none for wood free here),
/// the other inputs, the station, a knife, a launcher, then the time.
fn takes(content: &Content, level: &LevelState, player: &Actor, recipe: &RecipeDef) -> String {
    let mut words: Vec<String> = Vec::new();
    match recipe.uses {
        Some(RecipeUse::Temper) => words.push("a metal weapon or armour".into()),
        Some(RecipeUse::Whittle) => words.push("a wooden weapon".into()),
        Some(RecipeUse::Cut) => words.push("a loose gem".into()),
        Some(RecipeUse::Set) => words.push("an item and a cut gem".into()),
        Some(RecipeUse::Enchant(kind)) => {
            words.extend(super::enchant::takes(kind, recipe.crystals))
        }
        None => {}
    }
    let input_words = |input: &crate::content::RecipeInput| {
        let def = content.object(input.object()?);
        Some(match input.per_made {
            true => format!(
                "{} for each made",
                counted(&def.name, &def.plural, input.count)
            ),
            false => counted(&def.name, &def.plural, input.count),
        })
    };
    let mut sourced = Vec::new();
    if let Some(result) = &recipe.result {
        let batch = super::craft_make::batch_most(content, player, recipe);
        for part in &recipe.parts {
            match &part.source {
                PartSource::Stock => {
                    if super::fletch::part_free(content, level, player, recipe, part.part) {
                        continue;
                    }
                    let worked = worked_materials(content, recipe, result.object, part.part);
                    let object = result.object;
                    words.extend(stock_words(
                        content, recipe, object, part.part, &worked, batch,
                    ));
                }
                PartSource::Sources { input, stock, .. } => {
                    let mut either: Vec<String> = Vec::new();
                    let input = (recipe.inputs.iter())
                        .find(|i| i.object().is_some() && i.object() == *input);
                    if let Some(input) = input {
                        sourced.extend(input.object());
                        either.extend(input_words(input));
                    }
                    let object = result.object;
                    either.extend(stock_words(
                        content, recipe, object, part.part, stock, batch,
                    ));
                    if !either.is_empty() {
                        words.push(either.join(" or "));
                    }
                }
                PartSource::Gem => {}
            }
        }
    }
    if (recipe.parts.iter()).any(|p| p.source == PartSource::Gem) {
        words.push("a cut gem".into());
    }
    for input in &recipe.inputs {
        let Some(object) = input.object() else {
            words.extend(stock_input_words(content, input));
            continue;
        };
        if !sourced.contains(&object) {
            words.extend(input_words(input));
        }
    }
    if let Some(station) = recipe.station {
        let name = &content.terrain(station).name;
        words.push(format!("{} {name}", article(name)));
    }
    if recipe.knife {
        words.push("a knife".into());
    }
    if let Some(result) = recipe.result.as_ref().filter(|_| recipe.launcher) {
        let word = super::fletch::launcher_word(content, result.object);
        words.push(format!("{} {word} ready", article(word)));
    }
    let seconds = match recipe.uses {
        Some(RecipeUse::Temper | RecipeUse::Whittle) => {
            super::mastery::craft_seconds(content, player, content.temper().seconds)
        }
        Some(RecipeUse::Cut | RecipeUse::Set | RecipeUse::Enchant(_)) => {
            super::mastery::craft_seconds(content, player, recipe.seconds)
        }
        None => super::fletch::fletcher_seconds(content, player, recipe),
    };
    if seconds > 0 {
        words.push(format!(
            "about {}",
            content.english_word(Duration::seconds(u64::from(seconds)).spoken(), "bare")
        ));
    }
    words.join(", ")
}

/// The stock part `part` of `batch` of `object` takes, from `materials`, priced in the part's
/// base material where it is among them (else the cheapest of its group), once for each group
/// they hold, the base material's group first: "3 ingots", "7 billets or 7 shards", the
/// material named only where the part takes just that one of its group ("a block of flint";
/// [`crate::piece::any_names`]) (docs/design/professions.md §3: the part's share of the result's
/// weight, in pieces; another material of the group takes as many of its own).
fn stock_words(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
    materials: &[MaterialId],
    batch: u32,
) -> Option<String> {
    let base = content.object(object).parts.get(part)?;
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let group = |m: MaterialId| content.material(m).group.clone();
    // The base material's group first, each other group by its cheapest: a club is "7 billets
    // or 7 shards".
    let mut order = materials.to_vec();
    order.sort_by_key(|&m| group(m) != group(base.material));
    let mut groups: Vec<Option<String>> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    for &first in &order {
        if groups.contains(&group(first)) {
            continue;
        }
        groups.push(group(first));
        // Priced in the base material when it is of this group, else the piece's own (steel: a
        // wooden small shield in metal is 13 ingots, never adamantium's count), else the group's
        // cheapest.
        let named = |want: MaterialId| {
            (materials.iter().copied()).find(|&m| m == want && group(m) == group(first))
        };
        let material = (named(base.material))
            .or_else(|| piece.parts.first().and_then(|p| named(p.material)))
            .or_else(|| {
                (materials.iter().copied())
                    .filter(|&m| group(m) == group(first))
                    .min_by_key(|&m| content.material(m).value)
            })
            .unwrap_or(first);
        let count = super::craft_make::stock_need(content, recipe, part, material, batch)?;
        // The material named only where the part takes some of its group, the priced one first
        // (#1767).
        let mut of_group: Vec<MaterialId> = (materials.iter().copied())
            .filter(|&m| group(m) == group(first))
            .collect();
        of_group.sort_by_key(|&m| m != material);
        let (one, many) = crate::piece::any_names(content, piece, &of_group)?;
        words.push(counted(&one, &many, count));
    }
    (!words.is_empty()).then(|| words.join(" or "))
}

impl World {
    /// Whether the player is untrained in profession `skill`, saying so ("You are not trained
    /// in Metalcraft."): every recipe is its skill's (#1387), so its action is then refused,
    /// taking no time.
    pub(super) fn untrained(&mut self, skill: SkillId) -> bool {
        if self.state.player().skills.get(skill).is_some() {
            return false;
        }
        let name = self.content.skill_phrase(skill);
        self.say(Msg::new(msg::crafting::NOT_TRAINED).role("skill", name));
        true
    }

    /// The skill of built-in recipe `uses` (Temper's Metalcraft, Whittle's Woodcraft), if the
    /// content has it.
    pub(super) fn skill_of_use(&self, uses: RecipeUse) -> Option<SkillId> {
        self.content
            .recipes()
            .of_use(uses)
            .map(|recipe| recipe.skill)
    }

    /// The player makes a recipe (the module docs): a generic one is begun by
    /// [`World::start_making`]; a built-in or unknown one is refused, taking no time.
    pub(in crate::world) fn start_craft(&mut self, recipe: &str, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let Some(id) = content.recipes().id(recipe) else {
            self.say(msg::crafting::NO_SUCH_RECIPE);
            return Spent::Nothing;
        };
        let def = content.recipe(id);
        if !of_race(def, self.state.race_of(&content, self.state.player())) {
            self.say(msg::crafting::NO_SUCH_RECIPE);
            return Spent::Nothing;
        }
        if self.untrained(def.skill) {
            return Spent::Nothing;
        }
        let said = match def.uses {
            Some(RecipeUse::Cut) => return self.start_cut(id, picks),
            Some(RecipeUse::Set) => return self.start_set(id, picks),
            Some(RecipeUse::Enchant(_)) => return self.start_enchant(id, picks),
            Some(_) => Msg::new(msg::crafting::DONE_FROM_OWN_LIST)
                .role("recipe", content.recipe_phrase(&def.id)),
            None => return self.start_making(id, picks),
        };
        self.say(said);
        Spent::Nothing
    }
}
