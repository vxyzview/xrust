//! Bounded scrollback history of scrolled-out lines.

use crate::grid::Cell;

#[derive(Debug, Clone)]
pub struct Scrollback {
    lines: std::collections::VecDeque<Vec<Cell>>,
    /// Per-line flag: the line is a soft-wrap continuation of the NEXT row
    /// (or of the live screen, for the newest line).
    wraps: std::collections::VecDeque<bool>,
    capacity: usize,
    /// Monotonic counter of lines ever pushed.
    total_pushed: u64,
}

impl Scrollback {
    pub fn new(capacity: usize) -> Self {
        Scrollback {
            lines: std::collections::VecDeque::new(),
            wraps: std::collections::VecDeque::new(),
            capacity: capacity.max(1),
            total_pushed: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, cap: usize) {
        self.capacity = cap.max(1);
        while self.lines.len() > self.capacity {
            self.lines.pop_front();
            self.wraps.pop_front();
        }
    }

    /// Push a line at the back (most recent). `wraps_next` marks it as
    /// soft-wrapped into the following row. Trims from the front.
    pub fn push(&mut self, line: Vec<Cell>, wraps_next: bool) {
        if self.lines.len() == self.capacity {
            self.lines.pop_front();
            self.wraps.pop_front();
        }
        self.lines.push_back(line);
        self.wraps.push_back(wraps_next);
        self.total_pushed += 1;
    }

    /// Whether the newest history line soft-wraps into the live screen.
    pub fn last_wraps(&self) -> bool {
        self.wraps.back().copied().unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Line `i` counted from the oldest retained line.
    pub fn get(&self, i: usize) -> Option<&[Cell]> {
        self.lines.get(i).map(|v| v.as_slice())
    }

    pub fn wraps_at(&self, i: usize) -> Option<bool> {
        self.wraps.get(i).copied()
    }

    /// Last line (newest).
    pub fn last(&self) -> Option<&[Cell]> {
        self.lines.back().map(|v| v.as_slice())
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.wraps.clear();
        self.total_pushed = 0;
    }

    /// Absolute ordinal of the oldest retained line (for selection coords).
    pub fn base_ordinal(&self) -> u64 {
        self.total_pushed.saturating_sub(self.lines.len() as u64)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Cell]> {
        self.lines.iter().map(|v| v.as_slice())
    }

    /// Iterate (cells, wraps_next) pairs, oldest first.
    pub fn iter_with_flags(&self) -> impl Iterator<Item = (&[Cell], bool)> {
        self.lines
            .iter()
            .zip(self.wraps.iter())
            .map(|(c, w)| (c.as_slice(), *w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn line(tag: char) -> Vec<Cell> {
        (0..4)
            .map(|_| Cell {
                ch: tag,
                ..Cell::default()
            })
            .collect()
    }

    #[test]
    fn push_and_len() {
        let mut sb = Scrollback::new(5);
        for t in "abcde".chars() {
            sb.push(line(t), false);
        }
        assert_eq!(sb.len(), 5);
        assert_eq!(sb.get(0).unwrap()[0].ch, 'a');
        assert_eq!(sb.last().unwrap()[0].ch, 'e');
    }

    #[test]
    fn capacity_trim_oldest() {
        let mut sb = Scrollback::new(3);
        for t in "abcde".chars() {
            sb.push(line(t), false);
        }
        assert_eq!(sb.len(), 3);
        assert_eq!(sb.get(0).unwrap()[0].ch, 'c', "oldest lines trimmed first");
        assert_eq!(sb.base_ordinal(), 2, "2 lines were dropped");
    }

    #[test]
    fn shrink_capacity_drops_front() {
        let mut sb = Scrollback::new(10);
        for t in "abcde".chars() {
            sb.push(line(t), false);
        }
        sb.set_capacity(2);
        assert_eq!(sb.len(), 2);
        assert_eq!(sb.get(0).unwrap()[0].ch, 'd');
    }

    #[test]
    fn capacity_minimum_is_one() {
        let mut sb = Scrollback::new(0);
        sb.push(line('x'), false);
        assert_eq!(sb.len(), 1);
        sb.set_capacity(0);
        assert_eq!(sb.capacity(), 1);
    }

    #[test]
    fn clear_resets() {
        let mut sb = Scrollback::new(4);
        sb.push(line('a'), false);
        sb.push(line('b'), false);
        sb.clear();
        assert!(sb.is_empty());
        assert_eq!(sb.base_ordinal(), 0);
        assert!(!sb.last_wraps());
    }

    #[test]
    fn iteration_order_is_oldest_first() {
        let mut sb = Scrollback::new(5);
        for t in "xyz".chars() {
            sb.push(line(t), false);
        }
        let tags: Vec<char> = sb.iter().map(|l| l[0].ch).collect();
        assert_eq!(tags, vec!['x', 'y', 'z']);
    }

    #[test]
    fn wrap_flags_tracked_and_trimmed() {
        let mut sb = Scrollback::new(3);
        sb.push(line('a'), true);
        sb.push(line('b'), false);
        sb.push(line('c'), true);
        assert!(sb.last_wraps());
        assert_eq!(sb.wraps_at(0), Some(true));
        assert_eq!(sb.wraps_at(1), Some(false));
        sb.push(line('d'), false);
        assert_eq!(
            sb.wraps_at(0),
            Some(false),
            "oldest flag trimmed with its line"
        );
        let pairs: Vec<bool> = sb.iter_with_flags().map(|(_, w)| w).collect();
        assert_eq!(pairs, vec![false, true, false]);
    }

    #[test]
    fn wide_cells_survive_history() {
        let mut sb = Scrollback::new(2);
        let mut l = vec![Cell::default(); 4];
        l[0].ch = '中';
        l[0].attrs = crate::grid::ATTR_WIDE;
        l[1].attrs = crate::grid::ATTR_WIDE_CONT;
        sb.push(l, false);
        assert_eq!(sb.get(0).unwrap()[0].ch, '中');
        assert!(sb.get(0).unwrap()[1].is_wide_cont());
        assert_eq!(sb.get(0).unwrap()[1].fg, Color::Default);
    }

    #[test]
    fn get_out_of_range_is_none() {
        let sb = Scrollback::new(4);
        assert!(sb.get(0).is_none());
        assert!(sb.wraps_at(0).is_none());
    }
}
