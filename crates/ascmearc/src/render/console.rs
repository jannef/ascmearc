//! The CPU-side console: a grid of glyph cells that UI and map drawing write into. The renderer
//! uploads it to textures when it changes.
//!
//! A console is a layer: cells nothing was drawn into are transparent, so the layers below show
//! through. Each cell can also carry in-cell animation data and effects (glow, heat haze, the
//! depth blur) for the grid shader.

use ascmearc_core::color::Rgb;
use bevy::prelude::*;

/// One console cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb,
    pub bg: Rgb,
}

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: ' ',
        fg: Rgb::BLACK,
        bg: Rgb::BLACK,
    };
}

/// In-cell animation of one console cell, as stored in the per-cell animation texture (and, for
/// `flow`, the spare byte of the glyph texture).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellAnim {
    /// Entry in the grid animation table plus one; 0 is still.
    pub entry: u8,
    /// Phase in 1/256 cycles.
    pub phase: u8,
    /// Running water's flow on this cell ([`super::anim::flow_byte`]); 0 is none.
    pub flow: u8,
}

impl CellAnim {
    pub const STILL: CellAnim = CellAnim {
        entry: 0,
        phase: 0,
        flow: 0,
    };
}

/// Visual effects of one console cell, stored next to its animation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellFx {
    /// How much the glyph glows (HDR, picked up by bloom), in 1/255.
    pub glow: u8,
    /// Heat haze strength: the shader distorts this part of the screen, in 1/255.
    pub haze: u8,
}

impl CellFx {
    pub const NONE: CellFx = CellFx { glow: 0, haze: 0 };
}

/// The strongest depth blur a cell takes ([`Console::set_blur`] clamps to it): past it the pull
/// toward the background (`BLUR_PULL` in `console.wesl`) would wash the glyph out.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "draw_map's tower window view calls it (#1781)")
)]
pub const BLUR_MAX: u8 = 8;

/// A grid of cells; `None` cells are transparent.
#[derive(Resource, Debug, Default)]
pub struct Console {
    cols: u32,
    rows: u32,
    cells: Vec<Option<Cell>>,
    anims: Vec<CellAnim>,
    fx: Vec<CellFx>,
    blur: Vec<u8>,
    /// Set when cells change; cleared by the upload.
    pub dirty: bool,
}

impl Console {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    /// Cells in row-major order; `None` is transparent.
    pub fn cells(&self) -> &[Option<Cell>] {
        &self.cells
    }

    /// Per-cell animation, parallel to [`Console::cells`].
    pub fn anims(&self) -> &[CellAnim] {
        &self.anims
    }

    /// Per-cell effects, parallel to [`Console::cells`].
    pub fn fx(&self) -> &[CellFx] {
        &self.fx
    }

    /// Per-cell depth blur ([`Console::set_blur`]), parallel to [`Console::cells`].
    pub fn blur(&self) -> &[u8] {
        &self.blur
    }

    pub fn resize(&mut self, cols: u32, rows: u32) {
        if (cols, rows) != (self.cols, self.rows) {
            self.cols = cols;
            self.rows = rows;
            self.cells = vec![None; (cols * rows) as usize];
            self.anims = vec![CellAnim::STILL; (cols * rows) as usize];
            self.fx = vec![CellFx::NONE; (cols * rows) as usize];
            self.blur = vec![0; (cols * rows) as usize];
            self.dirty = true;
        }
    }

    /// Makes every cell transparent and still, without effects.
    pub fn clear(&mut self) {
        self.cells.fill(None);
        self.anims.fill(CellAnim::STILL);
        self.fx.fill(CellFx::NONE);
        self.blur.fill(0);
        self.dirty = true;
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as u32) < self.cols && (y as u32) < self.rows)
            .then(|| (y as u32 * self.cols + x as u32) as usize)
    }

    /// The cell at `(x, y)`; `None` if it is out of range or transparent.
    #[cfg(test)]
    pub fn get(&self, x: i32, y: i32) -> Option<Cell> {
        self.index(x, y).and_then(|i| self.cells[i])
    }

    /// Writes a still cell without effects; out-of-range positions are ignored.
    pub fn put(&mut self, x: i32, y: i32, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = Some(cell);
            self.anims[i] = CellAnim::STILL;
            self.fx[i] = CellFx::NONE;
            self.blur[i] = 0;
            self.dirty = true;
        }
    }

    /// Animates the cell at `(x, y)` in the grid shader.
    pub fn animate(&mut self, x: i32, y: i32, anim: CellAnim) {
        if let Some(i) = self.index(x, y) {
            self.anims[i] = anim;
            self.dirty = true;
        }
    }

    /// Sets the effects of the cell at `(x, y)`.
    pub fn set_fx(&mut self, x: i32, y: i32, fx: CellFx) {
        if let Some(i) = self.index(x, y)
            && self.fx[i] != fx
        {
            self.fx[i] = fx;
            self.dirty = true;
        }
    }

    /// Blurs the glyph at `(x, y)` as seen from `storey` storeys up (a tower's window view of
    /// the ground below; 0 is sharp, clamped to [`BLUR_MAX`]). The grid shader softens the glyph
    /// and pulls it toward the background, more per storey; the background and light stay put.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "draw_map's tower window view calls it (#1781)")
    )]
    pub fn set_blur(&mut self, x: i32, y: i32, storey: u8) {
        let blur = storey.min(BLUR_MAX);
        if let Some(i) = self.index(x, y)
            && self.blur[i] != blur
        {
            self.blur[i] = blur;
            self.dirty = true;
        }
    }

    /// Removes the glyph at `(x, y)` but keeps the background, e.g. where a glyph on the
    /// free-glyph layer stands in for it.
    pub fn clear_glyph(&mut self, x: i32, y: i32) {
        if let Some(i) = self.index(x, y)
            && let Some(cell) = &mut self.cells[i]
        {
            cell.ch = ' ';
            self.anims[i] = CellAnim::STILL;
            self.dirty = true;
        }
    }

    /// Writes `text` starting at `(x, y)` keeping each cell's background (black where the cell
    /// was transparent). Returns the number of cells written (clipped at the right edge).
    pub fn print(&mut self, x: i32, y: i32, text: &str, fg: Rgb) -> i32 {
        let mut written = 0;
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i as i32;
            if let Some(idx) = self.index(cx, y) {
                let cell = self.cells[idx].get_or_insert(Cell::BLANK);
                cell.ch = ch;
                cell.fg = fg;
                written += 1;
            }
        }
        self.dirty = true;
        written
    }

    /// Fills a rectangle with `cell`.
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, cell: Cell) {
        for cy in y..y + h {
            for cx in x..x + w {
                self.put(cx, cy, cell);
            }
        }
    }

    /// Draws a single-line box border with `bg` interior.
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, fg: Rgb, bg: Rgb) {
        if w < 2 || h < 2 {
            return;
        }
        self.fill(x, y, w, h, Cell { ch: ' ', fg, bg });
        let mut edge = |cx, cy, ch| self.put(cx, cy, Cell { ch, fg, bg });
        for cx in x + 1..x + w - 1 {
            edge(cx, y, '─');
            edge(cx, y + h - 1, '─');
        }
        for cy in y + 1..y + h - 1 {
            edge(x, cy, '│');
            edge(x + w - 1, cy, '│');
        }
        edge(x, y, '┌');
        edge(x + w - 1, y, '┐');
        edge(x, y + h - 1, '└');
        edge(x + w - 1, y + h - 1, '┘');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_clips_at_edges() {
        let mut c = Console::default();
        c.resize(5, 2);
        assert_eq!(c.print(3, 0, "hello", Rgb::WHITE), 2);
        assert_eq!(c.get(4, 0).map(|cell| cell.ch), Some('e'));
        assert_eq!(c.print(0, 5, "x", Rgb::WHITE), 0);
    }

    #[test]
    fn unwritten_cells_are_transparent() {
        let mut c = Console::default();
        c.resize(3, 1);
        assert_eq!(c.get(0, 0), None);
        c.print(0, 0, "a", Rgb::WHITE);
        assert_eq!(c.get(0, 0).map(|cell| cell.bg), Some(Rgb::BLACK));
        c.clear();
        assert!(c.cells().iter().all(Option::is_none));
    }

    #[test]
    fn animation_follows_the_cell() {
        let mut c = Console::default();
        c.resize(2, 1);
        let wave = CellAnim {
            entry: 1,
            phase: 9,
            flow: 0,
        };
        c.put(0, 0, Cell::BLANK);
        c.animate(0, 0, wave);
        assert_eq!(c.anims()[0], wave);
        // A new cell is still unless animated again; a cleared glyph stops animating.
        c.put(0, 0, Cell::BLANK);
        assert_eq!(c.anims()[0], CellAnim::STILL);
        c.animate(0, 0, wave);
        c.clear_glyph(0, 0);
        assert_eq!(c.anims()[0], CellAnim::STILL);
        assert_eq!(c.get(0, 0).map(|cell| cell.ch), Some(' '));
        // Clearing a transparent cell's glyph leaves it transparent.
        c.clear_glyph(1, 0);
        assert_eq!(c.get(1, 0), None);
    }

    #[test]
    fn effects_follow_the_cell() {
        let mut c = Console::default();
        c.resize(2, 1);
        let hot = CellFx {
            glow: 200,
            haze: 90,
        };
        c.put(0, 0, Cell::BLANK);
        c.set_fx(0, 0, hot);
        c.set_fx(5, 0, hot);
        assert_eq!(c.fx(), &[hot, CellFx::NONE]);
        c.put(0, 0, Cell::BLANK);
        assert_eq!(c.fx()[0], CellFx::NONE, "a new cell has no effects");
        c.set_fx(1, 0, hot);
        c.clear();
        assert!(c.fx().iter().all(|fx| *fx == CellFx::NONE));
    }

    #[test]
    fn blur_follows_the_cell() {
        let mut c = Console::default();
        c.resize(3, 1);
        c.put(0, 0, Cell::BLANK);
        c.set_blur(0, 0, 3);
        c.set_blur(1, 0, 200);
        c.set_blur(9, 0, 3);
        assert_eq!(c.blur(), &[3, BLUR_MAX, 0], "clamped, out of range ignored");
        c.put(0, 0, Cell::BLANK);
        assert_eq!(c.blur()[0], 0, "a new cell is sharp");
        c.set_fx(1, 0, CellFx { glow: 9, haze: 9 });
        assert_eq!(c.blur()[1], BLUR_MAX, "effects leave the blur alone");
        c.clear();
        assert!(c.blur().iter().all(|b| *b == 0));
    }

    /// The shader clamps the blur byte as `set_blur` does, and at the clamp the glyph still
    /// shows: the pull toward the background stays well short of the background itself.
    #[test]
    fn shader_constants_match() {
        let src = include_str!("console.wesl");
        let max = crate::render::wesl_const(src, "BLUR_MAX");
        assert_eq!(max, f32::from(BLUR_MAX), "BLUR_MAX");
        let pull = crate::render::wesl_const(src, "BLUR_PULL");
        assert!(pull * max <= 0.7, "BLUR_PULL × BLUR_MAX = {}", pull * max);
    }

    #[test]
    fn frame_draws_corners() {
        let mut c = Console::default();
        c.resize(4, 3);
        c.frame(0, 0, 4, 3, Rgb::WHITE, Rgb::BLACK);
        assert_eq!(c.get(0, 0).map(|cell| cell.ch), Some('┌'));
        assert_eq!(c.get(3, 2).map(|cell| cell.ch), Some('┘'));
        assert_eq!(c.get(1, 1).map(|cell| cell.ch), Some(' '));
    }
}
