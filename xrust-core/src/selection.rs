//! Selection over history + screen with copy semantics.

use crate::grid::Cell;

/// A position: `History(ordinal, col)` or `Screen(row, col)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pos {
    /// `ordinal` is the absolute line ordinal (see `Scrollback::base_ordinal`).
    History { ordinal: u64, col: usize },
    /// Row counted from the first visible row.
    Screen { row: usize, col: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: Option<Pos>,
    pub extent: Option<Pos>,
}

impl Selection {
    pub fn start(&mut self, p: Pos) {
        self.anchor = Some(p);
        self.extent = Some(p);
    }

    pub fn extend(&mut self, p: Pos) {
        if self.anchor.is_some() {
            self.extent = Some(p);
        }
    }

    pub fn clear(&mut self) {
        *self = Selection::default();
    }

    pub fn is_active(&self) -> bool {
        match (self.anchor, self.extent) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        }
    }

    fn ordered(&self) -> Option<(Pos, Pos)> {
        match (self.anchor, self.extent) {
            (Some(a), Some(b)) if a <= b => Some((a, b)),
            (Some(a), Some(b)) => Some((b, a)),
            _ => None,
        }
    }

    /// Copy the selected text. `history_line` maps an ordinal to the cells of
    /// that history line; `screen_line` maps a row to the cells of that row;
    /// `soft_wrap` for a screen row says whether the row is soft-wrapped
    /// into the next one (no newline on copy).
    #[allow(clippy::too_many_arguments)]
    pub fn copy_text(
        &self,
        history_line: &dyn Fn(u64) -> Option<Vec<Cell>>,
        screen_line: &dyn Fn(usize) -> Option<Vec<Cell>>,
        _screen_rows: usize,
        soft_wrap: &dyn Fn(usize) -> bool,
    ) -> Option<String> {
        let (a, b) = self.ordered()?;
        let mut out = String::new();
        let append_span =
            |cells: &[Cell], from: usize, to_incl: usize, add_nl: bool, out: &mut String| {
                let end = to_incl.min(cells.len().saturating_sub(1));
                let mut s = String::new();
                for cell in cells.iter().take(end + 1).skip(from) {
                    s.push(cell.ch);
                    s.push_str(&cell.combining);
                }
                out.push_str(&s);
                if add_nl {
                    out.push('\n');
                }
            };
        match (a, b) {
            (
                Pos::History {
                    ordinal: o1,
                    col: c1,
                },
                Pos::History {
                    ordinal: o2,
                    col: c2,
                },
            ) => {
                for o in o1..=o2 {
                    let line = history_line(o)?;
                    let hard = o < o2;
                    let (f, t) = if o1 == o2 {
                        (c1, c2)
                    } else if o == o1 {
                        (c1, usize::MAX)
                    } else if o == o2 {
                        (0, c2)
                    } else {
                        (0, usize::MAX)
                    };
                    append_span(&line, f, t, hard, &mut out);
                }
            }
            (Pos::Screen { row: r1, col: c1 }, Pos::Screen { row: r2, col: c2 }) => {
                for r in r1..=r2 {
                    let line = screen_line(r)?;
                    // Newline after an intermediate row only when that row
                    // hard-ends (i.e. it is not soft-wrapped into the next).
                    let hard = r < r2 && !soft_wrap(r);
                    let (f, t) = if r1 == r2 {
                        (c1, c2)
                    } else if r == r1 {
                        (c1, usize::MAX)
                    } else if r == r2 {
                        (0, c2)
                    } else {
                        (0, usize::MAX)
                    };
                    append_span(&line, f, t, hard, &mut out);
                }
            }
            (
                Pos::History {
                    ordinal: o1,
                    col: c1,
                },
                Pos::Screen { row: r2, col: c2 },
            ) => {
                // History part: from `o1` up to the newest retained line.
                let mut o = o1;
                let mut first = true;
                let mut guard = 0u64;
                while let Some(line) = history_line(o) {
                    let (f, t) = if first {
                        (c1, usize::MAX)
                    } else {
                        (0, usize::MAX)
                    };
                    append_span(&line, f, t, true, &mut out);
                    first = false;
                    o += 1;
                    guard += 1;
                    if guard > 1_000_000 {
                        break; // defensive: malformed history callback
                    }
                }
                // Screen part: rows 0..=r2.
                for r in 0..=r2 {
                    let line = screen_line(r)?;
                    let hard = r < r2 || !soft_wrap(r2);
                    let (f, t) = if r == r2 { (0, c2) } else { (0, usize::MAX) };
                    append_span(&line, f, t, hard, &mut out);
                }
            }
            (Pos::Screen { .. }, Pos::History { .. }) => {
                unreachable!("ordered() guarantees history first")
            }
        }
        Some(out)
    }
}

/// Word boundaries in a line for double-click selection: returns (start, end)
/// inclusive around `col`, splitting on whitespace and common delimiters.
pub fn word_bounds(line: &[Cell], col: usize) -> (usize, usize) {
    let is_delim = |c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '/' | '\\'
                    | '|'
                    | '"'
                    | '\''
                    | '`'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | ':'
                    | ';'
                    | ','
                    | '.'
                    | '@'
                    | '='
                    | '&'
                    | '#'
                    | '$'
                    | '!'
                    | '?'
                    | '*'
                    | '+'
                    | '~'
                    | '%'
            )
    };
    let ch_at = |i: usize| -> char { line.get(i).map(|c| c.ch).unwrap_or(' ') };
    let col = col.min(line.len().saturating_sub(1));
    if is_delim(ch_at(col)) {
        return (col, col);
    }
    let mut s = col;
    while s > 0 && !is_delim(ch_at(s - 1)) {
        s -= 1;
    }
    let mut e = col;
    while e + 1 < line.len() && !is_delim(ch_at(e + 1)) {
        e += 1;
    }
    (s, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|ch| Cell {
                ch,
                ..Cell::default()
            })
            .collect()
    }

    #[test]
    fn start_extend_clear() {
        let mut sel = Selection::default();
        assert!(!sel.is_active());
        sel.start(Pos::Screen { row: 0, col: 2 });
        assert!(!sel.is_active(), "single point is not a selection");
        sel.extend(Pos::Screen { row: 0, col: 5 });
        assert!(sel.is_active());
        sel.clear();
        assert!(sel.anchor.is_none());
    }

    #[test]
    fn extend_without_start_is_noop() {
        let mut sel = Selection::default();
        sel.extend(Pos::Screen { row: 1, col: 1 });
        assert!(sel.anchor.is_none());
    }

    #[test]
    fn ordering_is_normalized() {
        let mut sel = Selection::default();
        sel.start(Pos::Screen { row: 2, col: 4 });
        sel.extend(Pos::Screen { row: 0, col: 1 });
        let txt = sel
            .copy_text(
                &|_| None,
                &|r| match r {
                    0 => Some(cells("hello")),
                    1 => Some(cells("ab")),
                    2 => Some(cells("xyzw")),
                    _ => None,
                },
                4,
                &|_| false,
            )
            .unwrap();
        assert_eq!(
            txt, "ello\nab\nxyzw",
            "rows hard-end → newlines between them"
        );
    }

    #[test]
    fn screen_selection_single_row() {
        let mut sel = Selection::default();
        sel.start(Pos::Screen { row: 0, col: 1 });
        sel.extend(Pos::Screen { row: 0, col: 3 });
        let txt = sel
            .copy_text(&|_| None, &|_| Some(cells("abcdef")), 2, &|_| true)
            .unwrap();
        assert_eq!(txt, "bcd");
    }

    #[test]
    fn screen_selection_soft_wrap_no_trailing_newline() {
        let mut sel = Selection::default();
        sel.start(Pos::Screen { row: 0, col: 0 });
        sel.extend(Pos::Screen { row: 1, col: 2 });
        let txt = sel
            .copy_text(
                &|_| None,
                &|r| {
                    if r == 0 {
                        Some(cells("aaa"))
                    } else {
                        Some(cells("bbb"))
                    }
                },
                2,
                &|r| r == 0,
            )
            .unwrap();
        assert_eq!(txt, "aaabbb", "soft-wrapped row 0 has no newline");
    }

    #[test]
    fn history_selection_spans() {
        let lines: std::collections::HashMap<u64, Vec<Cell>> = [
            (0u64, cells("alpha")),
            (1, cells("beta")),
            (2, cells("gamma")),
        ]
        .into_iter()
        .collect();
        let mut sel = Selection::default();
        sel.start(Pos::History { ordinal: 0, col: 2 });
        sel.extend(Pos::History { ordinal: 2, col: 1 });
        let txt = sel
            .copy_text(&move |o| lines.get(&o).cloned(), &|_| None, 0, &|_| true)
            .unwrap();
        assert_eq!(txt, "pha\nbeta\nga");
    }

    #[test]
    fn word_bounds_inside_word() {
        let line = cells("foo bar.c");
        assert_eq!(word_bounds(&line, 1), (0, 2));
        assert_eq!(word_bounds(&line, 5), (4, 6), ".c splits the word");
        assert_eq!(word_bounds(&line, 8), (8, 8), "delim selects itself");
    }

    #[test]
    fn word_bounds_unicode() {
        let line = cells("中文 test");
        assert_eq!(word_bounds(&line, 1), (0, 1));
    }

    #[test]
    fn history_then_screen_returns_all() {
        let lines: std::collections::HashMap<u64, Vec<Cell>> =
            [(0u64, cells("HIST"))].into_iter().collect();
        let mut sel = Selection::default();
        sel.start(Pos::History { ordinal: 0, col: 0 });
        sel.extend(Pos::Screen { row: 1, col: 1 });
        let txt = sel
            .copy_text(
                &move |o| lines.get(&o).cloned(),
                &|r| {
                    if r == 0 {
                        Some(cells("s0"))
                    } else {
                        Some(cells("s1"))
                    }
                },
                2,
                &|_| true,
            )
            .unwrap();
        assert_eq!(txt, "HIST\ns0\ns1");
    }
}
