//! Screen grid model: cells, cursor, line operations, tab stops.
//!
//! The grid never touches scrollback: when a scroll must push a line into
//! history, `put_char` returns [`PutOutcome::RetryAfterScroll`] and the
//! caller (`term`) scrolls first, then re-issues the put.

use crate::color::Color;
use crate::wcwidth::char_width;

pub const ATTR_BOLD: u16 = 1 << 0;
pub const ATTR_ITALIC: u16 = 1 << 1;
pub const ATTR_UNDERLINE: u16 = 1 << 2;
pub const ATTR_STRIKE: u16 = 1 << 3;
pub const ATTR_REVERSE: u16 = 1 << 4;
pub const ATTR_INVISIBLE: u16 = 1 << 5;
pub const ATTR_DIM: u16 = 1 << 6;
pub const ATTR_WIDE: u16 = 1 << 7;
pub const ATTR_WIDE_CONT: u16 = 1 << 8;
pub const ATTR_BLINK: u16 = 1 << 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    /// Combining marks attached to `ch` (rendered in the same cell).
    pub combining: String,
    pub fg: Color,
    pub bg: Color,
    pub attrs: u16,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            combining: String::new(),
            fg: Color::Default,
            bg: Color::Default,
            attrs: 0,
        }
    }
}

impl Cell {
    pub fn is_blank(&self) -> bool {
        self.ch == ' '
            && self.combining.is_empty()
            && self.fg == Color::Default
            && self.bg == Color::Default
            && self.attrs == 0
    }

    pub fn is_wide(&self) -> bool {
        self.attrs & ATTR_WIDE != 0
    }

    pub fn is_wide_cont(&self) -> bool {
        self.attrs & ATTR_WIDE_CONT != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub saved_style: u32,
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub cols: usize,
    pub rows: usize,
    /// Row-major `rows * cols` cells.
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    /// DECSTBM scroll region, inclusive. `None` = full screen.
    pub scroll_region: Option<(usize, usize)>,
    /// Pending autowrap: cursor is conceptually past the last column.
    pub wrap_next: bool,
    /// Tab stop columns (bitmap of size cols).
    pub tabs: Vec<bool>,
    pub origin_mode: bool,
    /// Last printed char, for REP (CSI b).
    pub last_printed: Option<char>,
    /// Position of the most recently written cell (for combining marks).
    pub last_written: Option<(usize, usize)>,
    /// Per-row soft-wrap flag: row r's logical line continues into row r+1.
    pub wrapped: Vec<bool>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols >= 2 && rows >= 1, "grid too small");
        Grid {
            cols,
            rows,
            cells: vec![Cell::default(); rows * cols],
            cursor: Cursor::default(),
            scroll_region: None,
            wrap_next: false,
            tabs: Self::default_tabs(cols),
            origin_mode: false,
            last_printed: None,
            last_written: None,
            wrapped: vec![false; rows],
        }
    }

    fn default_tabs(cols: usize) -> Vec<bool> {
        let mut tabs = vec![false; cols];
        let mut i = 8;
        while i < cols {
            tabs[i] = true;
            i += 8;
        }
        tabs
    }

    pub fn row_mut(&mut self, r: usize) -> &mut [Cell] {
        let c = self.cols;
        &mut self.cells[r * c..(r + 1) * c]
    }

    pub fn row(&self, r: usize) -> &[Cell] {
        let c = self.cols;
        &self.cells[r * c..(r + 1) * c]
    }

    /// Top of scroll region. Cursor movement (CUU/CUD) clamps here
    /// regardless of origin mode; origin mode only affects CUP/VPA.
    pub fn region_top(&self) -> usize {
        match self.scroll_region {
            Some((t, _)) => t,
            None => 0,
        }
    }

    /// Bottom of scroll region, inclusive.
    pub fn region_bottom(&self) -> usize {
        match self.scroll_region {
            Some((_, b)) => b,
            None => self.rows - 1,
        }
    }

    // ---------- cursor movement ----------

    pub fn move_down(&mut self, n: usize) {
        self.cursor.row = (self.cursor.row + n).min(self.region_bottom());
    }

    pub fn move_up(&mut self, n: usize) {
        self.cursor.row = self.cursor.row.saturating_sub(n).max(self.region_top());
    }

    pub fn move_right(&mut self, n: usize) {
        self.cursor.col = (self.cursor.col + n).min(self.cols - 1);
        self.wrap_next = false;
    }

    pub fn move_left(&mut self, n: usize) {
        self.cursor.col = self.cursor.col.saturating_sub(n);
        self.wrap_next = false;
    }

    pub fn goto(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows - 1);
        self.cursor.col = col.min(self.cols - 1);
        self.wrap_next = false;
    }

    // ---------- printing ----------

    /// Advance the cursor one cell after printing at/before the last column,
    /// honoring DECAWM. Returns `true` when wrap-next became pending.
    pub fn advance(&mut self, wrap: bool) -> bool {
        if self.cursor.col + 1 < self.cols {
            self.cursor.col += 1;
            self.wrap_next = false;
            false
        } else if wrap {
            self.wrap_next = true;
            self.wrapped[self.cursor.row] = true;
            true
        } else {
            self.cursor.col = self.cols - 1;
            self.wrap_next = false;
            false
        }
    }

    /// Write `ch` (plus combining marks) honoring DECAWM and wide chars.
    /// Combining marks return `None` (no advance). Wide chars that would
    /// straddle the right edge wrap to the next row; if the cursor is on the
    /// region bottom row, returns `RetryAfterScroll` and writes nothing —
    /// the caller must scroll the region up once, reset the cursor to
    /// `(bottom, 0)`, and re-issue the put.
    #[allow(clippy::too_many_arguments)]
    pub fn put_char(
        &mut self,
        ch: char,
        combining: &str,
        fg: Color,
        bg: Color,
        attrs: u16,
        wrap: bool,
    ) -> PutOutcome {
        let width = char_width(ch);
        if width == 0 {
            // Attach combining marks to the last written cell (the cursor has
            // already advanced past the base character).
            if let Some((r, c)) = self.last_written {
                if let Some(cell) = self.cells.get_mut(r * self.cols + c) {
                    cell.combining.push_str(combining);
                    cell.combining.push(ch);
                }
            }
            return PutOutcome::None;
        }

        // Pending autowrap: move to start of the next row first.
        if self.wrap_next {
            self.wrap_next = false;
            if wrap {
                self.cursor.col = 0;
                self.cursor.row += 1;
                if self.cursor.row > self.region_bottom() {
                    self.cursor.row = self.region_bottom();
                    return PutOutcome::RetryAfterScroll;
                }
            } else {
                self.cursor.col = self.cols - 1;
            }
        }

        let (r, c) = (self.cursor.row, self.cursor.col);
        if width == 2 && c + 1 >= self.cols {
            // Wide char would straddle the right edge.
            if !wrap {
                // No autowrap: overwrite the last column, cursor stays.
                let cell = &mut self.cells[r * self.cols + c];
                *cell = Cell {
                    ch,
                    combining: combining.to_string(),
                    fg,
                    bg,
                    attrs: attrs | ATTR_WIDE,
                };
                self.last_printed = Some(ch);
                return PutOutcome::None;
            }
            // Wrap: write on the next row; scroll if at region bottom.
            self.wrapped[r] = true;
            self.cursor.col = 0;
            self.cursor.row += 1;
            if self.cursor.row > self.region_bottom() {
                self.cursor.row = self.region_bottom();
                return PutOutcome::RetryAfterScroll;
            }
            self.write_wide(self.cursor.row, 0, ch, combining, fg, bg, attrs);
            self.last_printed = Some(ch);
            return PutOutcome::Advanced;
        }

        if width == 2 {
            self.write_wide(r, c, ch, combining, fg, bg, attrs);
        } else {
            // If we land on a wide-continuation cell, blank the lead cell
            // to the left so the pair stays consistent.
            if c > 0 {
                let left = &mut self.cells[r * self.cols + c - 1];
                if left.is_wide() {
                    *left = Cell::default();
                }
            }
            let cell = &mut self.cells[r * self.cols + c];
            *cell = Cell {
                ch,
                combining: combining.to_string(),
                fg,
                bg,
                attrs,
            };
            // And clear a stale continuation to the right of the old wide lead.
            if c + 1 < self.cols && self.cells[r * self.cols + c + 1].is_wide_cont() {
                self.cells[r * self.cols + c + 1] = Cell::default();
            }
            self.last_written = Some((r, c));
        }
        self.last_printed = Some(ch);
        PutOutcome::Advanced
    }

    #[allow(clippy::too_many_arguments)]
    fn write_wide(
        &mut self,
        r: usize,
        c: usize,
        ch: char,
        combining: &str,
        fg: Color,
        bg: Color,
        attrs: u16,
    ) {
        let cols = self.cols;
        let base = r * cols + c;
        self.cells[base] = Cell {
            ch,
            combining: combining.to_string(),
            fg,
            bg,
            attrs: attrs | ATTR_WIDE,
        };
        self.cells[base + 1] = Cell {
            ch: ' ',
            combining: String::new(),
            fg: Color::Default,
            bg: Color::Default,
            attrs: ATTR_WIDE_CONT,
        };
        self.cursor.row = r;
        self.cursor.col = c + 1; // sit on the continuation cell; caller advances
        self.last_written = Some((r, c));
    }

    // ---------- line operations ----------

    pub fn erase_row(&mut self, r: usize) {
        let cols = self.cols;
        for cell in &mut self.cells[r * cols..(r + 1) * cols] {
            *cell = Cell::default();
        }
        self.wrapped[r] = false;
    }

    /// Scroll the region up `n`: content moves up, blank lines enter at the
    /// region bottom. Returns the lines scrolled out (top-down order).
    pub fn scroll_up(&mut self, n: usize) -> Vec<Vec<Cell>> {
        let top = self.region_top();
        let bottom = self.region_bottom();
        let n = n.min(bottom - top + 1);
        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            out.push(self.row(top + k).to_vec());
        }
        let cols = self.cols;
        for r in top..=bottom - n {
            let dst = r * cols;
            let src = (r + n) * cols;
            let moved: Vec<Cell> = self.cells[src..src + cols].to_vec();
            self.cells[dst..dst + cols].clone_from_slice(&moved);
        }
        for r in top..=bottom - n {
            self.wrapped[r] = self.wrapped[r + n];
        }
        for r in bottom - n + 1..=bottom {
            self.erase_row(r);
        }
        out
    }

    /// Scroll the region down `n`: blank lines enter at the region top.
    pub fn scroll_down(&mut self, n: usize) {
        let top = self.region_top();
        let bottom = self.region_bottom();
        let n = n.min(bottom - top + 1);
        let cols = self.cols;
        let mut r = bottom;
        while r > top + n - 1 {
            let dst = r * cols;
            let src = (r - n) * cols;
            let moved: Vec<Cell> = self.cells[src..src + cols].to_vec();
            self.cells[dst..dst + cols].clone_from_slice(&moved);
            if r == 0 {
                break;
            }
            r -= 1;
        }
        for r in (top + n..=bottom).rev() {
            self.wrapped[r] = self.wrapped[r - n];
        }
        for r in top..top + n {
            self.erase_row(r);
        }
    }

    /// ED (Erase in Display): 0 = cursor→end, 1 = start→cursor, 2 = all.
    pub fn erase_display(&mut self, mode: u8) {
        let (r, c) = (self.cursor.row, self.cursor.col);
        match mode {
            0 => {
                self.erase_row_range(r, c, self.cols);
                for rr in r + 1..self.rows {
                    self.erase_row(rr);
                }
            }
            1 => {
                for rr in 0..r {
                    self.erase_row(rr);
                }
                self.erase_row_range(r, 0, c + 1);
            }
            _ => {
                for rr in 0..self.rows {
                    self.erase_row(rr);
                }
            }
        }
    }

    /// EL (Erase in Line): 0 = cursor→end, 1 = start→cursor, 2 = whole line.
    pub fn erase_line(&mut self, mode: u8) {
        let (r, c) = (self.cursor.row, self.cursor.col);
        match mode {
            0 => self.erase_row_range(r, c, self.cols),
            1 => self.erase_row_range(r, 0, c + 1),
            _ => self.erase_row(r),
        }
    }

    fn erase_row_range(&mut self, r: usize, from: usize, to: usize) {
        let cols = self.cols;
        let row = &mut self.cells[r * cols..(r + 1) * cols];
        for cell in row[from.min(cols)..to.min(cols)].iter_mut() {
            *cell = Cell::default();
        }
    }

    /// ICH: insert `n` blanks at the cursor, shifting the rest right.
    pub fn insert_cells(&mut self, n: usize) {
        let (r, c) = (self.cursor.row, self.cursor.col);
        let cols = self.cols;
        let n = n.min(cols - c);
        let base = r * cols;
        let shifted: Vec<Cell> = self.cells[base + c..base + cols - n].to_vec();
        self.cells[base + c + n..base + cols].clone_from_slice(&shifted);
        for cell in &mut self.cells[base + c..base + c + n] {
            *cell = Cell::default();
        }
    }

    /// DCH: delete `n` cells at the cursor, shifting the rest left.
    pub fn delete_cells(&mut self, n: usize) {
        let (r, c) = (self.cursor.row, self.cursor.col);
        let cols = self.cols;
        let n = n.min(cols - c);
        let base = r * cols;
        let shifted: Vec<Cell> = self.cells[base + c + n..base + cols].to_vec();
        self.cells[base + c..base + c + shifted.len()].clone_from_slice(&shifted);
        for cell in &mut self.cells[base + cols - n..base + cols] {
            *cell = Cell::default();
        }
    }

    /// IL: insert `n` blank lines inside the region at the cursor row.
    pub fn insert_lines(&mut self, n: usize) {
        let r = self.cursor.row;
        if r < self.region_top() || r > self.region_bottom() {
            return;
        }
        let bottom = self.region_bottom();
        let saved = self.scroll_region;
        self.scroll_region = Some((r, bottom));
        self.scroll_down(n);
        self.scroll_region = saved;
    }

    /// DL: delete `n` lines at the cursor row inside the region.
    pub fn delete_lines(&mut self, n: usize) {
        let r = self.cursor.row;
        if r < self.region_top() || r > self.region_bottom() {
            return;
        }
        let bottom = self.region_bottom();
        let saved = self.scroll_region;
        self.scroll_region = Some((r, bottom));
        self.scroll_up(n);
        self.scroll_region = saved;
    }

    pub fn set_region(&mut self, top: usize, bottom: usize) {
        if top < bottom && bottom < self.rows {
            self.scroll_region = Some((top, bottom));
        }
    }

    pub fn set_region_to_full(&mut self) {
        self.scroll_region = None;
    }

    // ---------- tabs ----------

    pub fn tab_forward(&mut self) {
        let col = self.cursor.col;
        let next = (col + 1..self.cols)
            .find(|&c| self.tabs[c])
            .unwrap_or(self.cols - 1);
        self.cursor.col = next;
        self.wrap_next = false;
    }

    pub fn tab_backward(&mut self) {
        let col = self.cursor.col;
        let prev = (0..col).rev().find(|&c| self.tabs[c]).unwrap_or(0);
        self.cursor.col = prev;
        self.wrap_next = false;
    }

    pub fn set_tab(&mut self) {
        self.tabs[self.cursor.col] = true;
    }

    pub fn clear_tab(&mut self, mode: u8) {
        match mode {
            0 => self.tabs[self.cursor.col] = false,
            3 => self.tabs.iter_mut().for_each(|t| *t = false),
            _ => {}
        }
    }

    /// One row as text; wide-continuation cells are skipped (their glyph is
    /// kept in the lead cell); trailing blanks trimmed unless `keep_trailing`.
    /// A fully blank row yields "".
    pub fn row_text(&self, r: usize, keep_trailing: bool) -> String {
        let row = self.row(r);
        let mut s = String::new();
        let take = if keep_trailing {
            self.cols
        } else {
            row.iter()
                .rposition(|c| !c.is_blank() && !c.is_wide_cont())
                .map_or(0, |i| i + 1)
        };
        for cell in row.iter().take(take) {
            if cell.is_wide_cont() {
                continue;
            }
            s.push(cell.ch);
            s.push_str(&cell.combining);
        }
        s
    }

    /// Resize; content preserved as-is, cursor clamped, stale region reset.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut cells = vec![Cell::default(); rows * cols];
        let copy_cols = self.cols.min(cols);
        let copy_rows = self.rows.min(rows);
        for r in 0..copy_rows {
            let src = r * self.cols;
            let dst = r * cols;
            cells[dst..dst + copy_cols].clone_from_slice(&self.cells[src..src + copy_cols]);
        }
        self.cols = cols;
        self.rows = rows;
        self.cells = cells;
        self.wrapped = vec![false; rows];
        self.tabs = Self::default_tabs(cols);
        self.cursor.col = self.cursor.col.min(cols - 1);
        self.cursor.row = self.cursor.row.min(rows - 1);
        if let Some((t, b)) = self.scroll_region {
            if t >= rows || b >= rows || t >= b {
                self.scroll_region = None;
            }
        }
    }
}

/// Result of `Grid::put_char`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOutcome {
    /// Combining mark attached; cursor did not move.
    None,
    /// Char written; caller must call `advance`.
    Advanced,
    /// Char not written: the caller must scroll the region up once (pushing
    /// the top line to scrollback), reset the cursor to `(bottom, 0)`,
    /// clear wrap-next, and re-issue `put_char`.
    RetryAfterScroll,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(g: &mut Grid, ch: char) {
        loop {
            match g.put_char(ch, "", Color::Default, Color::Default, 0, true) {
                PutOutcome::Advanced => {
                    g.advance(true);
                    return;
                }
                PutOutcome::None => return,
                PutOutcome::RetryAfterScroll => {
                    g.scroll_up(1);
                    g.cursor.row = g.region_bottom();
                    g.cursor.col = 0;
                    g.wrap_next = false;
                }
            }
        }
    }

    #[test]
    fn new_grid_blank_and_tab_stops() {
        let g = Grid::new(20, 6);
        assert_eq!(g.row_text(0, false), "");
        assert!(g.tabs[8] && g.tabs[16]);
        assert!(!g.tabs[7]);
    }

    #[test]
    fn print_and_read_back() {
        let mut g = Grid::new(10, 3);
        for ch in "hello".chars() {
            put(&mut g, ch);
        }
        assert_eq!(g.row_text(0, false), "hello");
        assert_eq!((g.cursor.row, g.cursor.col), (0, 5));
    }

    #[test]
    fn cursor_moves_clamped_to_region() {
        let mut g = Grid::new(20, 10);
        g.set_region(2, 7);
        g.goto(7, 0);
        g.move_down(5);
        assert_eq!(g.cursor.row, 7, "must clamp to region bottom");
        g.goto(2, 0);
        g.move_up(5);
        assert_eq!(g.cursor.row, 2, "must clamp to region top");
    }

    #[test]
    fn origin_mode_limits_goto_and_region() {
        let mut g = Grid::new(10, 6);
        g.origin_mode = true;
        g.set_region(2, 5);
        g.goto(5, 0); // absolute CUP still clamps to screen, not region
        assert_eq!(g.cursor.row, 5);
        g.move_up(10);
        assert_eq!(g.cursor.row, 2, "origin mode clamps up-moves to region top");
        assert_eq!(g.region_top(), 2);
    }

    #[test]
    fn autowrap_and_wrap_pending() {
        let mut g = Grid::new(4, 3);
        for ch in "abcdef".chars() {
            put(&mut g, ch);
        }
        assert_eq!(g.row_text(0, false), "abcd");
        assert_eq!(g.row_text(1, false), "ef");
        assert_eq!(g.cursor.col, 2);
        assert!(!g.wrap_next);
    }

    #[test]
    fn autowrap_scrolls_at_bottom() {
        let mut g = Grid::new(4, 2);
        for ch in "aaaabbbbcccc".chars() {
            put(&mut g, ch);
        }
        assert_eq!(g.row_text(0, false), "bbbb", "aaaa scrolled out");
        assert_eq!(g.row_text(1, false), "cccc");
        assert_eq!(g.cursor.row, 1);
    }

    #[test]
    fn no_autowrap_overwrites_last_column() {
        let mut g = Grid::new(4, 2);
        for ch in "abcdef".chars() {
            let _ = g.put_char(ch, "", Color::Default, Color::Default, 0, false);
            g.advance(false);
        }
        assert_eq!(g.row_text(0, false), "abcf");
        assert_eq!(g.cursor.col, 3);
        assert!(!g.wrap_next);
    }

    #[test]
    fn wide_char_uses_two_cells() {
        let mut g = Grid::new(6, 2);
        put(&mut g, '中');
        put(&mut g, '文');
        assert_eq!(g.row_text(0, false), "中文");
        assert_eq!(g.cursor.col, 4);
        assert!(g.row(0)[0].is_wide());
        assert!(g.row(0)[1].is_wide_cont());
    }

    #[test]
    fn wide_char_straddling_edge_wraps() {
        let mut g = Grid::new(5, 2);
        for ch in "ab中".chars() {
            put(&mut g, ch);
        }
        assert_eq!(g.row_text(0, false), "ab中");
        assert_eq!(g.cursor.col, 4);
    }

    #[test]
    fn wide_char_at_last_column_wraps_to_next_row() {
        let mut g = Grid::new(4, 2);
        for ch in "abc中".chars() {
            put(&mut g, ch);
        }
        assert_eq!(g.row_text(0, false), "abc", "existing row content intact");
        assert_eq!(g.row_text(1, false), "中");
    }

    #[test]
    fn wide_char_overwrites_continuation_cleanly() {
        let mut g = Grid::new(4, 1);
        put(&mut g, '中'); // cells 0-1 wide+cont, cursor ends at col 2
        g.move_left(1); // cursor on the continuation cell
        let out = g.put_char('x', "", Color::Default, Color::Default, 0, true);
        assert!(matches!(out, PutOutcome::Advanced));
        assert!(g.row(0)[0].is_blank(), "stale wide lead must be cleared");
        assert_eq!(g.row_text(0, false), " x", "col0 blank, col1 holds 'x'");
    }

    #[test]
    fn combining_mark_attaches_to_previous() {
        let mut g = Grid::new(10, 2);
        put(&mut g, 'e');
        let out = g.put_char('\u{0301}', "", Color::Default, Color::Default, 0, true);
        assert!(matches!(out, PutOutcome::None));
        assert_eq!(
            g.row_text(0, false),
            "e\u{0301}",
            "mark attaches to the written cell, not the cursor cell"
        );
        assert_eq!(g.cursor.col, 1);
    }

    #[test]
    fn erase_display_modes() {
        let mut g = Grid::new(6, 4);
        for ch in "abcdefgh".chars() {
            put(&mut g, ch);
        }
        g.goto(1, 2);
        g.erase_display(0);
        assert_eq!(g.row_text(0, false), "abcdef");
        assert_eq!(
            g.row_text(1, false),
            "gh",
            "cursor was past 'gh'; only blanks erased"
        );
        assert_eq!(g.row_text(2, false), "");

        let mut g2 = Grid::new(6, 4);
        for ch in "abcdefgh".chars() {
            put(&mut g2, ch);
        }
        g2.goto(1, 2);
        g2.erase_display(1);
        assert_eq!(g2.row_text(0, false), "");
        assert_eq!(
            g2.row_text(1, false),
            "",
            "ED1 erases through the cursor: g,h are before col 2"
        );
        assert_eq!(g2.row_text(2, false), "");
    }

    #[test]
    fn erase_line_modes() {
        let mut g = Grid::new(8, 2);
        for ch in "12345678".chars() {
            put(&mut g, ch);
        }
        g.goto(0, 3);
        g.erase_line(0);
        assert_eq!(g.row_text(0, false), "123");
        let mut g2 = Grid::new(8, 2);
        for ch in "12345678".chars() {
            put(&mut g2, ch);
        }
        g2.goto(0, 3);
        g2.erase_line(1);
        assert_eq!(
            g2.row_text(0, false),
            "    5678",
            "leading blanks are part of the line"
        );
        let mut g3 = Grid::new(8, 2);
        for ch in "12345678".chars() {
            put(&mut g3, ch);
        }
        g3.erase_line(2);
        assert_eq!(g3.row_text(0, false), "");
    }

    #[test]
    fn scroll_up_returns_scrolled_lines() {
        let mut g = Grid::new(4, 3);
        for r in 0..3 {
            g.goto(r, 0);
            put(&mut g, (b'0' + r as u8) as char);
        }
        let out = g.scroll_up(1);
        assert_eq!(out.len(), 1);
        assert_eq!(String::from_iter(out[0].iter().map(|c| c.ch)).trim(), "0");
        assert_eq!(g.row_text(0, false), "1");
        assert_eq!(g.row_text(2, false), "");
    }

    #[test]
    fn scroll_down_only_in_region() {
        let mut g = Grid::new(4, 5);
        for r in 0..5 {
            g.goto(r, 0);
            put(&mut g, (b'0' + r as u8) as char);
        }
        g.set_region(1, 3);
        g.scroll_down(2);
        assert_eq!(g.row_text(0, false), "0", "outside region untouched");
        assert_eq!(g.row_text(1, false), "", "top of region blanked");
        assert_eq!(g.row_text(2, false), "", "shift stops at region top");
        assert_eq!(g.row_text(3, false), "1", "old row 1 moved down 2");
        assert_eq!(g.row_text(4, false), "4", "outside region untouched");
    }

    #[test]
    fn insert_delete_cells_shift() {
        let mut g = Grid::new(8, 2);
        for ch in "abcdef".chars() {
            put(&mut g, ch);
        }
        g.goto(0, 2);
        g.insert_cells(2);
        assert_eq!(g.row_text(0, false), "ab  cdef");
        g.goto(0, 2);
        g.delete_cells(2);
        assert_eq!(g.row_text(0, false), "abcdef");
        g.goto(0, 0);
        g.delete_cells(100);
        assert_eq!(g.row_text(0, false), "");
    }

    #[test]
    fn insert_delete_lines_respect_cursor_position() {
        let mut g = Grid::new(6, 5);
        for r in 0..4 {
            g.goto(r, 0);
            put(&mut g, (b'A' + r as u8) as char);
        }
        g.goto(1, 0);
        g.insert_lines(1);
        assert_eq!(g.row_text(1, false), "");
        assert_eq!(g.row_text(2, false), "B");
        g.delete_lines(1);
        assert_eq!(g.row_text(1, false), "B");
        assert_eq!(g.row_text(2, false), "C");
    }

    #[test]
    fn insert_lines_preserves_outer_region() {
        let mut g = Grid::new(6, 6);
        g.set_region(1, 4);
        g.goto(2, 0);
        g.insert_lines(1);
        assert_eq!(
            g.scroll_region,
            Some((1, 4)),
            "outer region must be restored"
        );
    }

    #[test]
    fn insert_lines_outside_region_is_noop() {
        let mut g = Grid::new(6, 5);
        g.set_region(2, 4);
        g.goto(0, 0);
        put(&mut g, 'X');
        g.insert_lines(2);
        assert_eq!(g.row_text(0, false), "X", "row outside region untouched");
    }

    #[test]
    fn tabs_forward_backward_clear() {
        let mut g = Grid::new(40, 2);
        g.goto(0, 3);
        g.tab_forward();
        assert_eq!(g.cursor.col, 8);
        g.tab_forward();
        assert_eq!(g.cursor.col, 16);
        g.tab_backward();
        assert_eq!(g.cursor.col, 8);
        // No custom tab before column 8 → backtab lands at 0.
        g.tab_backward();
        assert_eq!(g.cursor.col, 0);
        g.goto(0, 5);
        g.set_tab();
        g.goto(0, 3);
        g.tab_forward();
        assert_eq!(g.cursor.col, 5, "custom tab found before the default one");
        g.goto(0, 8);
        g.clear_tab(0);
        g.goto(0, 9);
        g.tab_backward();
        assert_eq!(g.cursor.col, 5);
        g.clear_tab(3);
        g.goto(0, 2);
        g.tab_forward();
        assert_eq!(g.cursor.col, 39, "no tabs left → last column");
    }

    #[test]
    fn resize_preserves_content_and_clamps_cursor() {
        let mut g = Grid::new(8, 4);
        for ch in "hello".chars() {
            put(&mut g, ch);
        }
        g.resize(4, 6);
        assert_eq!(g.row_text(0, false), "hell");
        assert!(g.cursor.col < 4);
        assert_eq!(g.rows, 6);
        assert!(!g.tabs[0]);
        assert_eq!(g.tabs.len(), 4);
    }

    #[test]
    fn resize_resets_stale_region() {
        let mut g = Grid::new(8, 6);
        g.set_region(2, 5);
        g.resize(8, 3);
        assert_eq!(
            g.scroll_region, None,
            "region must not survive shrink past it"
        );
    }

    #[test]
    fn reverse_video_flag_roundtrip() {
        let mut g = Grid::new(4, 1);
        put(&mut g, 'x');
        assert_eq!(g.row(0)[0].attrs & ATTR_REVERSE, 0);
        g.row_mut(0)[0].attrs |= ATTR_REVERSE;
        assert!(g.row(0)[0].attrs & ATTR_REVERSE != 0);
    }

    #[test]
    fn wrapped_flags_track_soft_wrap() {
        let mut g = Grid::new(4, 3);
        for ch in "abcdefgh".chars() {
            put(&mut g, ch);
        }
        assert!(g.wrapped[0], "row 0 wraps into row 1 after 'e' moved down");
        assert!(g.wrapped[1], "row 1 is full with wrap pending as well");
        assert!(!g.wrapped[2]);
        g.erase_row(0);
        assert!(!g.wrapped[0], "erasing a row clears its flag");
    }
}
