//! Scrollback re-flow on terminal resize.
//!
//! Every history row carries a `wraps_next` flag (see `Scrollback::push`):
//! `true` means the row is a soft-wrap continuation of the row that follows
//! it (or of the live screen, for the newest row). Re-flow rebuilds logical
//! lines from those flags and re-wraps them at the new width. Wide-char
//! lead/continuation pairs are never split.

use crate::grid::{Cell, ATTR_WIDE, ATTR_WIDE_CONT};

/// One logical line: cells plus whether it ended with a hard newline.
#[derive(Debug, Clone, PartialEq)]
struct Logical {
    cells: Vec<Cell>,
    hard_end: bool,
}

/// Merge rows into logical lines using per-row wrap flags.
fn to_logical(rows: &[(Vec<Cell>, bool)]) -> Vec<Logical> {
    let mut out: Vec<Logical> = Vec::new();
    for (row, wraps_next) in rows {
        match out.last_mut() {
            Some(prev) if !prev.hard_end => {
                for c in row.iter() {
                    if c.is_wide_cont() {
                        continue;
                    }
                    prev.cells.push(c.clone());
                }
                if !*wraps_next {
                    prev.hard_end = true;
                }
            }
            _ => {
                let cells: Vec<Cell> = row.iter().filter(|c| !c.is_wide_cont()).cloned().collect();
                out.push(Logical {
                    cells,
                    hard_end: !*wraps_next,
                });
            }
        }
    }
    out
}

/// Wrap a logical line at `width`. Never splits a wide pair: when a wide
/// lead would overflow, the whole pair moves to the next row.
fn wrap_logical(cells: &[Cell], width: usize) -> Vec<Vec<Cell>> {
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    let mut cur: Vec<Cell> = Vec::with_capacity(width);
    let mut used = 0usize;
    for cell in cells {
        let w = if cell.is_wide() { 2 } else { 1 };
        if used + w > width {
            rows.push(std::mem::take(&mut cur));
            used = 0;
        }
        cur.push(cell.clone());
        used += w;
        // (continuation markers rebuilt by `normalize_wide_markers`)
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows
}

/// Re-flow history rows for a new width.
///
/// `rows` — old history rows with their wrap flags, oldest first;
/// `new_width` — target width. Returns `(new_rows, new_flags)` where the
/// flags follow the same convention: `true` on a row means it soft-wraps
/// into the row after it (the final `true` continues onto the live screen).
pub fn reflow(rows: &[(Vec<Cell>, bool)], new_width: usize) -> (Vec<Vec<Cell>>, Vec<bool>) {
    if new_width == 0 {
        return (Vec::new(), Vec::new());
    }
    let logical = to_logical(rows);
    let mut out_rows: Vec<Vec<Cell>> = Vec::new();
    let mut out_flags: Vec<bool> = Vec::new();
    for l in &logical {
        if l.cells.is_empty() {
            out_rows.push(Vec::new());
            out_flags.push(false);
            continue;
        }
        let wrapped = wrap_logical(&l.cells, new_width);
        let n = wrapped.len();
        for (i, row) in wrapped.into_iter().enumerate() {
            let is_last_row = i == n - 1;
            out_flags.push(if is_last_row { !l.hard_end } else { true });
            out_rows.push(row);
        }
    }
    (out_rows, out_flags)
}

/// Recompute `ATTR_WIDE_CONT` markers from `ATTR_WIDE` leads after surgery.
pub fn normalize_wide_markers(rows: &mut [Vec<Cell>]) {
    for row in rows.iter_mut() {
        let mut expect_cont = false;
        for cell in row.iter_mut() {
            if expect_cont {
                if cell.is_wide() {
                    cell.attrs = (cell.attrs & !ATTR_WIDE) | ATTR_WIDE_CONT;
                } else {
                    cell.attrs |= ATTR_WIDE_CONT;
                }
                expect_cont = false;
            } else if cell.is_wide() {
                expect_cont = true;
            } else {
                cell.attrs &= !ATTR_WIDE_CONT;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn cells(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|ch| Cell {
                ch,
                ..Cell::default()
            })
            .collect()
    }

    fn text(row: &[Cell]) -> String {
        row.iter().map(|c| c.ch).collect()
    }

    /// Build owned row data for reflow input.
    fn rows_of(spec: &[(&str, bool)]) -> Vec<(Vec<Cell>, bool)> {
        spec.iter().map(|(s, w)| (cells(s), *w)).collect()
    }

    #[test]
    fn shrink_wraps_long_line() {
        // Old width 4: "abcdefgh" as "abcd"(wraps) | "efgh"(continues on screen).
        let rows = rows_of(&[("abcd", true), ("efgh", true)]);
        let (out, flags) = reflow(&rows, 3);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["abc", "def", "gh"]);
        assert_eq!(
            flags,
            vec![true, true, true],
            "newest row still continues onto screen"
        );
    }

    #[test]
    fn grow_unwraps() {
        let rows = rows_of(&[("ab", true), ("cd", true)]);
        let (out, flags) = reflow(&rows, 4);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["abcd"]);
        assert_eq!(flags, vec![true]);
    }

    #[test]
    fn hard_newlines_respected() {
        let rows = rows_of(&[("abcd", false), ("ef", true)]);
        let (out, flags) = reflow(&rows, 2);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["ab", "cd", "ef"]);
        assert_eq!(flags, vec![true, false, true]);
    }

    #[test]
    fn grow_merges_only_soft_wraps() {
        let rows = rows_of(&[("ab", true), ("cd", false), ("xy", false)]);
        let (out, flags) = reflow(&rows, 4);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["abcd", "xy"]);
        assert_eq!(flags, vec![false, false]);
    }

    #[test]
    fn wide_pairs_never_split_on_shrink() {
        let mut l = cells("ab");
        let mut wide = Cell {
            ch: '中',
            ..Cell::default()
        };
        wide.attrs |= ATTR_WIDE;
        l.push(wide);
        let mut cont = Cell {
            ch: ' ',
            ..Cell::default()
        };
        cont.attrs |= ATTR_WIDE_CONT;
        l.push(cont);
        l.extend(cells("cd"));
        let rows = vec![(l, false)];
        let (out, _) = reflow(&rows, 3);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["ab", "中c", "d"]);
    }

    #[test]
    fn empty_history_reflows_to_empty() {
        let (out, flags) = reflow(&[], 10);
        assert!(out.is_empty());
        assert!(flags.is_empty());
    }

    #[test]
    fn zero_width_is_safe() {
        let rows = vec![(cells("abc"), false)];
        let (out, _) = reflow(&rows, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn normalize_markers_rebuilds_continuations() {
        let mut row = cells("ab");
        let mut wide = Cell {
            ch: '中',
            ..Cell::default()
        };
        wide.attrs |= ATTR_WIDE;
        row.push(wide);
        row.push(Cell {
            ch: 'c',
            ..Cell::default()
        });
        let mut rows = vec![row];
        normalize_wide_markers(&mut rows);
        let r = &rows[0];
        assert!(r[2].is_wide());
        assert!(r[3].is_wide_cont(), "marker rebuilt");
    }

    #[test]
    fn shrink_then_grow_roundtrip() {
        let rows = rows_of(&[("abcd", true), ("efgh", true)]);
        let (shrunk, flags) = reflow(&rows, 3);
        assert_eq!(flags, vec![true, true, true]);
        // Grow pass: the first two new rows still continue; the last ends.
        let view: Vec<(Vec<Cell>, bool)> = vec![
            (shrunk[0].clone(), true),
            (shrunk[1].clone(), true),
            (shrunk[2].clone(), false),
        ];
        let (grown, flags2) = reflow(&view, 8);
        let t: Vec<String> = grown.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["abcdefgh"]);
        assert_eq!(flags2, vec![false]);
    }

    #[test]
    fn colors_survive_reflow() {
        let mut row = cells("abcdef");
        for c in row.iter_mut().take(2) {
            c.fg = Color::Rgb(0xFF0000);
        }
        let rows = vec![(row, false)];
        let (out, _) = reflow(&rows, 3);
        assert_eq!(out[0][0].fg, Color::Rgb(0xFF0000));
        assert_eq!(out[0][2].fg, Color::Default);
        assert_eq!(out[1][0].fg, Color::Default);
    }

    #[test]
    fn flags_survive_shrink_of_multi_line_logical() {
        let rows = rows_of(&[("abcd", true), ("efgh", true)]);
        let (out, flags) = reflow(&rows, 5);
        let t: Vec<String> = out.iter().map(|r| text(r)).collect();
        assert_eq!(t, vec!["abcde", "fgh"]);
        assert_eq!(flags, vec![true, true]);
    }
}
