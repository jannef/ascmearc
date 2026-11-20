//! The forging wizard (professions; docs/design/professions.md §8; #1335):
//! what a generic recipe's row on a profession's screen opens. One paged list for each choice of
//! the recipe, in the order the rules give them (`PerceptionView::craft_steps`): the stock for
//! each chosen part ("blade"; the arrows' head: arrowheads, then shards and blocks, after a
//! **None** row that makes them brittle), each input, then the ornament (the cut gems carried,
//! after a **None** row). A row is a stack as the inventory lists it, with how many the
//! recipe takes of it ("uses 3", an arrowhead "uses 1 each"); a stack too small is greyed "too
//! few" and cannot be picked. The header carries the recipe's name and the running choice ("Long sword — blade: steel"). A
//! letter on the page, or Enter on the `>` row, picks and goes to the next step. An optional
//! step's None row is a row like the others: Up from the list's first row (Down from its last)
//! moves the cursor onto it, Enter on it picks None, and so does `-`; the cursor starts on it
//! when it is the only row (#1778). Esc goes back a step, its pick kept (the list opens on
//! it), and closes the wizard from the first. The last pick opens the one dialog
//! (`ScreenName::CraftConfirm`), which asks with what the rules say of the picks; declined, it
//! closes back to the last step; accepted, it applies `Action::Craft` and closes the wizard and the profession's screen
//! beneath, back to the map (#1769). Every key is fixed.
//!
//! An enchanting's later steps follow its earlier picks (`PerceptionView::craft_steps_given`,
//! read again after each pick, [`CraftPickScreen::reread`]): the seals list the families of the
//! crystals picked in words, the affix kept the item's affixes; a seal's None skips the seals
//! after it (and Esc skips back over them).
//!
//! Its words are `assets/lang/en/ui/craft_pick_screen.toml` (`T::…`, [`super::text`]); the
//! recipe's, the steps', the stacks', and the plan's are core's.

use super::dialog_screen::Dialog;
use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::{ScreenAction, ScreenName};
use super::text::{craft_pick_screen as T, ui, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use ascmearc_core::Action;
use ascmearc_core::content::{ItemClassId, RecipeId};
use ascmearc_core::lang::Arg;
use ascmearc_core::perception::PerceptionView;
use ascmearc_core::world::{CraftOption, CraftStep, CraftStepKind};

/// Panel width: room for a stack's listed name with what the recipe takes of it.
const WIDTH: i32 = 64;

/// The dialog the wizard's last pick opens (`ScreenName::CraftConfirm`): "Forge a
/// ruby-pommelled long sword? It uses 3 steel ingots and the ruby, at the anvil, and takes about
/// 1 hour.", the profession's verb its title and its yes; the rules' reason, and only Back, when
/// the picks do not hold. Its no closes it, back to the last step.
pub fn craft_dialog(
    run: Option<&PerceptionView<'_>>,
    craft: Option<(RecipeId, Vec<Option<usize>>)>,
) -> Dialog {
    let (Some(view), Some((recipe, picks))) = (run, craft) else {
        return Dialog::new(ui(T::DIALOG_TITLE), ui(T::DIALOG_NOTHING))
            .cancel(ui(T::DIALOG_BACK), ScreenAction::Close);
    };
    let plan = match view.craft_plan(recipe, &picks) {
        Ok(plan) => plan,
        Err(why) => {
            return Dialog::new(ui(T::DIALOG_TITLE), why)
                .cancel(ui(T::DIALOG_BACK), ScreenAction::Close);
        }
    };
    let mut args = vec![
        ("verb", Arg::from(plan.verb.clone())),
        ("name", plan.name.into()),
        ("time", plan.time.into()),
    ];
    // A work that uses nothing up and needs no station (Enchanting's Unbind) says only its time.
    let uses = !plan.uses.is_empty();
    if uses {
        args.push(("uses", plan.uses.into()));
    }
    let at = plan.station.is_some();
    if let Some(station) = plan.station {
        args.push(("station", station.into()));
    }
    let text = match (uses, at) {
        (false, false) => T::DIALOG_TAKES,
        (false, true) => T::DIALOG_TAKES_AT,
        (true, false) => T::DIALOG_USES,
        (true, true) => T::DIALOG_USES_AT,
    };
    let text = ui_with(text, &args);
    let action = Action::Craft {
        recipe: view.content().recipe(recipe).id.clone(),
        picks,
    };
    Dialog::yes_no(&plan.verb, text, &plan.verb, action, ui(T::DIALOG_NOT_NOW))
}

#[derive(Debug)]
pub struct CraftPickScreen {
    recipe: Option<RecipeId>,
    /// The recipe's name: "Long sword".
    name: String,
    steps: Vec<CraftStep>,
    /// The step shown.
    step: usize,
    /// A pick for every step, kept when going back (`None`: not made, or None chosen).
    picks: Vec<Option<usize>>,
    /// Each step's stacks, grouped by item class.
    lists: Vec<PagedList<CraftOption>>,
    /// For each step, whether the cursor is on its None row rather than in its list.
    on_none: Vec<bool>,
    /// The steps with no pick made, to tell a changed pack from picks that narrowed them.
    base: Vec<CraftStep>,
    /// A pick was made since the steps were read: they are read again ([`Self::reread`]).
    stale: bool,
}

impl CraftPickScreen {
    /// The wizard of `recipe`, read from the player's perception (`None` outside a run: empty);
    /// `recipe` `None` is the first recipe with a choice (`--screen craft-pick`).
    pub fn new(view: Option<&PerceptionView<'_>>, recipe: Option<RecipeId>) -> Self {
        let mut screen = Self {
            recipe,
            name: String::new(),
            steps: Vec::new(),
            step: 0,
            picks: Vec::new(),
            lists: Vec::new(),
            on_none: Vec::new(),
            base: Vec::new(),
            stale: false,
        };
        if let Some(view) = view {
            screen.read(view);
        }
        screen
    }

    /// Reads the steps again from `view` (the first recipe with a choice when none is set): on
    /// the first step, no pick made.
    fn read(&mut self, view: &PerceptionView<'_>) {
        let content = view.content();
        let recipes = content.recipes();
        let recipe = self.recipe.or_else(|| {
            let generic = (recipes.list.iter().enumerate()).filter(|(_, r)| r.result.is_some());
            generic
                .map(|(i, _)| RecipeId(i as u16))
                .find(|&id| !view.craft_steps(id).is_empty())
        });
        let Some(recipe) = recipe else {
            return;
        };
        self.recipe = Some(recipe);
        self.name = content.recipe(recipe).name.clone();
        self.steps = view.craft_steps(recipe);
        self.base = self.steps.clone();
        self.stale = false;
        self.picks = vec![None; self.steps.len()];
        self.step = 0;
        self.lists = (0..self.steps.len())
            .map(|i| self.list_of(view, i))
            .collect();
        self.on_none = vec![false; self.steps.len()];
        if !self.steps.is_empty() {
            self.go_to(0);
        }
    }

    /// The pack changed (the work done): when the steps differ, the wizard starts over on them.
    pub fn refresh(&mut self, view: &PerceptionView<'_>) {
        let Some(recipe) = self.recipe else {
            return;
        };
        if view.craft_steps(recipe) != self.base {
            self.read(view);
        }
    }

    /// Whether a pick was made since the steps were read.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Reads the steps again as the picks made narrow them (the module docs): a step whose
    /// options changed gets its new list, and loses a pick no longer among them.
    pub fn reread(&mut self, view: &PerceptionView<'_>) {
        let Some(recipe) = self.recipe.filter(|_| self.stale) else {
            return;
        };
        self.stale = false;
        let steps = view.craft_steps_given(recipe, &self.picks);
        if steps.len() != self.steps.len() {
            return;
        }
        for (i, step) in steps.into_iter().enumerate() {
            if step == self.steps[i] {
                continue;
            }
            if (self.picks[i]).is_some_and(|at| !step.options.iter().any(|o| o.at == at)) {
                self.picks[i] = None;
            }
            self.steps[i] = step;
            self.lists[i] = self.list_of(view, i);
        }
        self.go_to(self.step);
    }

    /// The list of the step shown.
    fn list(&self) -> &PagedList<CraftOption> {
        &self.lists[self.step.min(self.lists.len().saturating_sub(1))]
    }

    /// The list of step `step`'s stacks, grouped by their item class.
    fn list_of(&self, view: &PerceptionView<'_>, step: usize) -> PagedList<CraftOption> {
        let Some(step) = self.steps.get(step) else {
            return PagedList::empty();
        };
        if matches!(step.kind, CraftStepKind::Family | CraftStepKind::Affix) {
            // Not stacks: one group, headed by what they are.
            let header = match step.kind {
                CraftStepKind::Family => ui(T::HEADING_FAMILIES),
                _ => ui(T::HEADING_MAGIC),
            };
            let items = (step.options.iter().cloned())
                .map(|o| (ItemClassId(0), o))
                .collect();
            return PagedList::grouped(items, |_| header.clone());
        }
        let content = view.content();
        let pack = view.inventory();
        let items = (step.options.iter())
            .filter_map(|o| Some((content.object(pack.get(o.at)?.object).class, o.clone())))
            .collect();
        PagedList::new(items, content)
    }

    /// The recipe the wizard makes, and the picks so far, in step order (the dialog's).
    pub fn chosen(&self) -> Option<(RecipeId, Vec<Option<usize>>)> {
        Some((self.recipe?, self.picks.clone()))
    }

    /// Lays the list out for a console of `cols` by `rows`: the frame, the margin, the header
    /// and None lines, the gaps, and the footer leave the rest to a page.
    pub fn fit(&self, _cols: u32, rows: u32) {
        for list in &self.lists {
            list.set_room((rows as i32 - 10).max(1) as usize);
        }
    }

    /// Applies a key press (the module docs).
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back {
            if self.step == 0 {
                return ScreenAction::Close;
            }
            // Back over seals skipped by a None, to the first of them.
            let mut to = self.step - 1;
            while to > 0 && self.skipped_seal(to) && self.skipped_seal(to - 1) {
                to -= 1;
            }
            self.go_to(to);
            return ScreenAction::Stay;
        }
        let Some(step) = self.steps.get(self.step) else {
            return match press.nav {
                Nav::Enter => ScreenAction::Close,
                _ => ScreenAction::Stay,
            };
        };
        if press.text == Some('-') {
            return match step.optional {
                true => self.pick(None),
                false => ScreenAction::Stay,
            };
        }
        let step = self.step;
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.lists[step].at_letter(c) {
                Some(row) => {
                    self.on_none[step] = false;
                    self.lists[step].select(row);
                    self.choose(row)
                }
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter if self.on_none[step] => self.pick(None),
            Nav::Enter => self.choose(self.lists[step].cursor()),
            nav => {
                self.navigate(nav);
                ScreenAction::Stay
            }
        }
    }

    /// Moves the cursor of the step shown: through its list, and on an optional step onto and
    /// off its None row, which stands above the list's first row (and wraps round below its
    /// last).
    fn navigate(&mut self, nav: Nav) {
        let step = self.step;
        let list = &mut self.lists[step];
        if !self.steps[step].optional || list.is_empty() {
            list.navigate(nav);
            return;
        }
        let last = list.len() - 1;
        let on_none = &mut self.on_none[step];
        match nav {
            Nav::Up if *on_none => {
                *on_none = false;
                list.select(last);
            }
            Nav::Down if *on_none => {
                *on_none = false;
                list.select(0);
            }
            Nav::Up if list.cursor() == 0 => *on_none = true,
            Nav::Down if list.cursor() == last => *on_none = true,
            Nav::NextPage | Nav::PrevPage if list.page_count() > 1 => {
                *on_none = false;
                list.navigate(nav);
            }
            nav => {
                if !*on_none {
                    list.navigate(nav);
                }
            }
        }
    }

    /// Picks list item `row`, if the stack holds enough.
    fn choose(&mut self, row: usize) -> ScreenAction {
        match self.list().get(row).filter(|o| o.enough) {
            Some(option) => self.pick(Some(option.at)),
            None => ScreenAction::Stay,
        }
    }

    /// Whether step `step` is a seal with None picked.
    fn skipped_seal(&self, step: usize) -> bool {
        self.steps[step].kind == CraftStepKind::Family && self.picks[step].is_none()
    }

    /// Makes the step's pick and goes on: to the next step, or after the last to the dialog. A
    /// seal's None is None for the seals after it too, which are passed over.
    fn pick(&mut self, at: Option<usize>) -> ScreenAction {
        self.picks[self.step] = at;
        self.on_none[self.step] = at.is_none();
        self.stale = true;
        let mut next = self.step + 1;
        if at.is_none() && self.steps[self.step].kind == CraftStepKind::Family {
            while next < self.steps.len() && self.steps[next].kind == CraftStepKind::Family {
                self.picks[next] = None;
                self.on_none[next] = true;
                next += 1;
            }
        }
        if next < self.steps.len() {
            self.go_to(next);
            return ScreenAction::Stay;
        }
        ScreenAction::Open(ScreenName::CraftConfirm)
    }

    /// Shows step `step`, the cursor on its pick if it has one, and on its None row when that is
    /// the only row.
    fn go_to(&mut self, step: usize) {
        self.step = step;
        let list = &mut self.lists[step];
        let row = (self.picks[step]).and_then(|at| list.iter().position(|o| o.at == at));
        if let Some(row) = row {
            list.select(row);
            self.on_none[step] = false;
        }
        let optional = self.steps[step].optional;
        if !optional || list.is_empty() {
            self.on_none[step] = optional;
        }
    }

    pub fn draw(&self, console: &mut Console) {
        self.fit(console.cols(), console.rows());
        let count = self.list().tallest().max(1) as i32;
        let Some(step) = self.steps.get(self.step) else {
            let title = format!(" {} ", ui(T::TITLE_EMPTY));
            let panel = Panel::new(console, &title, WIDTH, 6);
            let none = ui(T::NOTHING_TO_CHOOSE);
            console.print_in(
                panel.inner_x(),
                panel.y + 2,
                &none,
                TEXT_DIM,
                panel.inner_w(),
            );
            panel.footer(console, &ui(T::ESC_BACK));
            return;
        };
        let name = || ("name", Arg::from(self.name.clone()));
        let label = ("label", Arg::from(step.label.clone()));
        let title = format!(" {} ", ui_with(T::TITLE_STEP, &[name(), label]));
        let panel = Panel::new(console, &title, WIDTH, count + 9);
        self.list().draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 2;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        // The running choice: every step before the one shown.
        let chosen: Vec<String> = (self.steps.iter().zip(&self.picks))
            .take(self.step)
            .map(|(s, pick)| {
                let what = (pick.and_then(|at| s.options.iter().find(|o| o.at == at)))
                    .map_or("—".to_string(), |o| o.short.clone());
                let args = [("label", s.label.clone().into()), ("what", what.into())];
                ui_with(T::CHOSEN_STEP, &args)
            })
            .collect();
        let header = match chosen.is_empty() {
            true => self.name.clone(),
            false => ui_with(
                T::HEADER_CHOSEN,
                &[name(), ("chosen", chosen.join(", ").into())],
            ),
        };
        let header: String = header
            .chars()
            .take(panel.inner_w().max(0) as usize)
            .collect();
        console.print(x, panel.y + 2, &header, TEXT_DIM);
        let mut y = panel.y + 4;
        let on_none = self.on_none[self.step];
        if step.optional && fits(y) {
            let color = if on_none { ACCENT } else { TEXT };
            if on_none {
                console.print(x, y, ">", ACCENT);
            }
            console.print_in(x + 2, y, &ui(T::NONE_ROW), color, panel.inner_w() - 2);
            y += 1;
        }
        if self.list().is_empty() && fits(y) {
            let none = match (step.optional, step.kind) {
                (true, CraftStepKind::Ornament) => T::EMPTY_ORNAMENT,
                (true, CraftStepKind::Family) => T::EMPTY_FAMILY,
                (true, CraftStepKind::Affix) => T::EMPTY_AFFIX,
                (true, _) => T::EMPTY_HEAD,
                (false, _) => T::EMPTY_REQUIRED,
            };
            console.print_in(x + 2, y, &ui(none), TEXT_DIM, panel.inner_w() - 2);
        }
        for (row, y) in self.list().rows().into_iter().zip(y..) {
            if !fits(y) {
                break;
            }
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(option) = self.list().get(index) else {
                continue;
            };
            let selected = !on_none && index == self.list().cursor();
            let color = match (option.enough, selected) {
                (false, _) => TEXT_DIM,
                (true, true) => ACCENT,
                (true, false) => TEXT,
            };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            let takes = || [("n", Arg::Number(i64::from(option.takes)))];
            let state = match (step.kind, option.enough, option.each) {
                (CraftStepKind::Family, _, _) => ui(T::STATE_CRYSTAL),
                (CraftStepKind::Affix, _, _) => ui(T::STATE_KEPT),
                (_, true, true) => ui_with(T::STATE_USES_EACH, &takes()),
                (_, true, false) => ui_with(T::STATE_USES, &takes()),
                (_, false, _) => ui(T::STATE_TOO_FEW),
            };
            let state_x = right - state.chars().count() as i32;
            let room = (state_x - x - 3).max(0) as usize;
            let name: String = format!("{letter} - {}", option.name)
                .chars()
                .take(room)
                .collect();
            console.print(x + 2, y, &name, color);
            console.print(state_x, y, &state, TEXT_DIM);
        }
        let keys = HINTS;
        let confirm = ("confirm", Arg::from(keys.confirm));
        let mut hints = Vec::new();
        match self.list().last_letter() {
            Some('a') => hints.push(ui_with(T::CHOOSE_A, &[confirm])),
            Some(last) => {
                let last = ("last", last.to_string().into());
                hints.push(ui_with(T::CHOOSE_LETTERS, &[last, confirm]));
            }
            None if step.optional => hints.push(ui_with(T::CHOOSE_NONE, &[confirm])),
            None => {}
        }
        if step.optional {
            hints.push(ui(T::NONE_KEY));
        }
        if let Some(page) = self.list().hint(&keys) {
            hints.push(page);
        }
        hints.push(ui(T::ESC_BACK));
        panel.footer(console, &hints.join("   "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::World;
    use ascmearc_core::content::Content;
    use std::sync::Arc;

    /// The forge with its stock picked up, beside the anvil.
    fn world() -> World {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut world = World::new(content, "forge", None).expect("world");
        for action in ascmearc_core::action::parse_actions("move:E pickup").expect("parses") {
            world.apply(&action);
        }
        world
    }

    #[test]
    fn letters_pick_esc_goes_back_keeping_the_pick_and_the_last_pick_asks() {
        let world = world();
        let view = world.perception();
        let sword = view
            .content()
            .recipes()
            .id("short_sword")
            .expect("short sword");
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        screen.fit(100, 60);
        let keys = Bindings::default();
        // The blade is required: `-` does nothing; the mithril (`c`) is picked.
        assert_eq!(screen.handle(&Press::typed('-'), &keys), ScreenAction::Stay);
        assert_eq!(screen.step, 0);
        assert_eq!(screen.handle(&Press::typed('c'), &keys), ScreenAction::Stay);
        assert_eq!(screen.step, 1);
        // Back to the blade, the cursor on the mithril; on again, then None for the pommel.
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &keys),
            ScreenAction::Stay
        );
        assert_eq!(screen.step, 0);
        let on = screen.list().selected().map(|o| o.short.clone());
        assert_eq!(on.as_deref(), Some("mithril"));
        screen.handle(&Press::nav(Nav::Enter), &keys);
        assert_eq!(
            screen.handle(&Press::typed('-'), &keys),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
        let (recipe, picks) = screen.chosen().expect("chosen");
        assert_eq!((recipe, picks.len()), (sword, 2));
        assert_eq!(picks[1], None);
        // The dialog names what will be made and how.
        let dialog = craft_dialog(Some(&view), screen.chosen());
        assert_eq!(dialog.title, "Forge");
        assert_eq!(
            dialog.text,
            "Forge a mithril short sword? It uses 2 mithril ingots, at the anvil, and takes \
             about 1 hour."
        );
        // Esc on the first step closes the wizard.
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &keys),
            ScreenAction::Close
        );
    }

    #[test]
    fn the_none_row_is_a_cursor_row_that_enter_picks() {
        let world = world();
        let view = world.perception();
        let sword = view
            .content()
            .recipes()
            .id("short_sword")
            .expect("short sword");
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        screen.fit(100, 60);
        let keys = Bindings::default();
        screen.handle(&Press::typed('c'), &keys);
        assert_eq!(screen.step, 1);
        assert!(!screen.list().is_empty() && !screen.on_none[1]);
        // Up from the first gem onto None, Down back off it; Up again and Enter picks None.
        screen.handle(&Press::nav(Nav::Up), &keys);
        assert!(screen.on_none[1]);
        screen.handle(&Press::nav(Nav::Down), &keys);
        assert!(!screen.on_none[1] && screen.list().cursor() == 0);
        screen.handle(&Press::nav(Nav::Up), &keys);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &keys),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
        assert_eq!(screen.chosen().expect("chosen").1[1], None);
        // Back to the pommel: the cursor is on the None picked.
        screen.handle(&Press::nav(Nav::Back), &keys);
        assert!(screen.on_none[1]);
    }

    #[test]
    fn a_none_only_step_starts_on_none() {
        let world = world();
        let view = world.perception();
        let sword = view
            .content()
            .recipes()
            .id("short_sword")
            .expect("short sword");
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        screen.lists[1] = PagedList::empty();
        screen.handle(&Press::typed('c'), &Bindings::default());
        assert!(screen.on_none[1]);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &Bindings::default()),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
    }

    /// A wizard trained in Enchanting with a long sword and four red crystals picked up.
    fn enchanter() -> World {
        let mut files = ascmearc_assets::files();
        files.edit_table("classes", |text| {
            text.replace(
                "enchanting = { chance = 30,",
                "enchanting = { chance = 100,",
            )
        });
        files.insert(
            "levels/test_seal_ui.toml",
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             [[item]]\nat = [2, 1]\nobject = \"long_sword\"\n\
             [[item]]\nat = [2, 1]\nobject = \"crystal_red\"\ncount = 4\n",
        );
        files.insert(
            "scenarios/test_seal_ui.toml",
            "name = \"T\"\nstart_level = \"test_seal_ui\"\n\
             character = { race = \"human\", gender = \"female\", class = \"wizard\" }\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut world = World::new(content, "test_seal_ui", Some(5)).expect("world");
        for action in ascmearc_core::action::parse_actions("move:E pickup").expect("parses") {
            world.apply(&action);
        }
        world
    }

    /// Picks the row of the step shown whose short name is `short`, then reads the steps again
    /// (as `route_keys` does after a press).
    fn choose_short(screen: &mut CraftPickScreen, view: &PerceptionView<'_>, short: &str) {
        let row = (screen.list().iter())
            .position(|o| o.short == short)
            .unwrap_or_else(|| panic!("{short} in step {}", screen.step));
        screen.lists[screen.step].select(row);
        screen.choose(row);
        screen.reread(view);
    }

    #[test]
    fn the_seals_follow_the_crystal_and_none_skips_the_rest() {
        let world = enchanter();
        let view = world.perception();
        let kindle = view.content().recipes().id("kindle").expect("kindle");
        let mut screen = CraftPickScreen::new(Some(&view), Some(kindle));
        screen.fit(100, 60);
        let labels: Vec<&str> = screen.steps.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(
            labels,
            ["item", "crystal", "seal", "second seal", "third seal"]
        );
        choose_short(&mut screen, &view, "long sword");
        choose_short(&mut screen, &view, "red crystal");
        assert_eq!(screen.step, 2);
        assert_eq!(screen.steps[2].kind, CraftStepKind::Family);
        let words: Vec<String> = screen.list().iter().map(|o| o.name.clone()).collect();
        assert!(
            !words.is_empty() && !words.iter().any(|w| w == "mana"),
            "{words:?}"
        );
        // The first seal is not offered again; None then skips the third.
        let first = words[0].clone();
        choose_short(&mut screen, &view, &first);
        assert_eq!(screen.step, 3);
        assert!(!screen.list().iter().any(|o| o.name == first));
        let keys = Bindings::default();
        assert_eq!(
            screen.handle(&Press::typed('-'), &keys),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
        let (_, picks) = screen.chosen().expect("chosen");
        assert_eq!(picks.len(), 5);
        assert!(picks[2].is_some() && picks[3].is_none() && picks[4].is_none());
        let dialog = craft_dialog(Some(&view), screen.chosen());
        assert!(
            dialog
                .text
                .contains(&format!("one more red crystal, sealing out {first}")),
            "{}",
            dialog.text
        );
        // Esc from the skipped third seal goes back over it to the first one skipped.
        screen.go_to(4);
        screen.handle(&Press::nav(Nav::Back), &keys);
        assert_eq!(screen.step, 3);
    }
}
