//! Terminal character width (subset of POSIX wcwidth) and wide-char helpers.
//!
//! Covers: zero-width combining marks, East Asian Wide/Fullwidth ranges,
//! and emoji ranges common in shells (git prompts, `ls` output).

/// Width in terminal cells: 0 for combining marks, 2 for wide/fullwidth,
/// 1 otherwise. Control chars (< 0x20) are handled by the caller (parser),
/// not here.
pub fn char_width(c: char) -> usize {
    let cp = c as u32;
    if cp == 0 {
        return 0;
    }
    if in_ranges(cp, ZERO_WIDTH) {
        return 0;
    }
    if in_ranges(cp, WIDE) {
        return 2;
    }
    1
}

const fn in_ranges(cp: u32, ranges: &[(u32, u32)]) -> bool {
    let mut i = 0;
    while i < ranges.len() {
        if cp >= ranges[i].0 && cp <= ranges[i].1 {
            return true;
        }
        i += 1;
    }
    false
}

/// Combining marks + zero-width (compact, common ranges).
const ZERO_WIDTH: &[(u32, u32)] = &[
    (0x0300, 0x036F),
    (0x0483, 0x0489),
    (0x0591, 0x05C7),
    (0x0610, 0x061A),
    (0x064B, 0x065F),
    (0x0670, 0x0670),
    (0x06D6, 0x06DC),
    (0x0730, 0x074A),
    (0x07A6, 0x07B0),
    (0x0900, 0x0903),
    (0x093A, 0x093C),
    (0x0941, 0x0948),
    (0x0E31, 0x0E31),
    (0x0E34, 0x0E3A),
    (0x0E47, 0x0E4E),
    (0x1AB0, 0x1AFF),
    (0x1DC0, 0x1DFF),
    (0x200B, 0x200F),
    (0x20D0, 0x20F0),
    (0xFE00, 0xFE0F),
    (0xFE20, 0xFE2F),
    (0xFEFF, 0xFEFF),
    (0x1D167, 0x1D169),
    (0xE0100, 0xE01EF),
];

/// East Asian Wide/Fullwidth + emoji (common shell-visible ranges).
const WIDE: &[(u32, u32)] = &[
    (0x1100, 0x115F),
    (0x2329, 0x232A),
    (0x2E80, 0x303E),
    (0x3041, 0x33FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA000, 0xA4CF),
    (0xA960, 0xA97F),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE10, 0xFE19),
    (0xFE30, 0xFE6B),
    (0xFF00, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x16FE0, 0x16FE4),
    (0x17000, 0x187F7),
    (0x1B000, 0x1B152),
    (0x1F004, 0x1F004),
    (0x1F0CF, 0x1F0CF),
    (0x1F18E, 0x1F18E),
    (0x1F191, 0x1F19A),
    (0x1F200, 0x1F320),
    (0x1F32D, 0x1F335),
    (0x1F337, 0x1F37C),
    (0x1F37E, 0x1F393),
    (0x1F3A0, 0x1F3CA),
    (0x1F3CF, 0x1F3D3),
    (0x1F3E0, 0x1F3F0),
    (0x1F3F4, 0x1F3F4),
    (0x1F3F8, 0x1F43E),
    (0x1F440, 0x1F440),
    (0x1F442, 0x1F4FC),
    (0x1F4FF, 0x1F53D),
    (0x1F54B, 0x1F54E),
    (0x1F550, 0x1F567),
    (0x1F57A, 0x1F57A),
    (0x1F595, 0x1F596),
    (0x1F5A4, 0x1F5A4),
    (0x1F5FB, 0x1F64F),
    (0x1F680, 0x1F6C5),
    (0x1F6CC, 0x1F6CC),
    (0x1F6D0, 0x1F6D2),
    (0x1F6EB, 0x1F6EC),
    (0x1F6F4, 0x1F6FA),
    (0x1F7E0, 0x1F7EB),
    (0x1F90D, 0x1F971),
    (0x1F973, 0x1F976),
    (0x1F97A, 0x1F9A2),
    (0x1F9A5, 0x1F9AA),
    (0x1F9AE, 0x1F9CA),
    (0x1F9CD, 0x1F9FF),
    (0x1FA70, 0x1FA95),
    (0x20000, 0x2FFFD),
    (0x30000, 0x3FFFD),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one_cell() {
        for c in ['a', 'Z', '0', ' ', '~', '!'] {
            assert_eq!(char_width(c), 1);
        }
    }

    #[test]
    fn combining_is_zero_cell() {
        for c in ['\u{0301}', '\u{05BF}', '\u{20E3}', '\u{FE0F}'] {
            assert_eq!(char_width(c), 0);
        }
    }

    #[test]
    fn cjk_and_kana_are_two_cells() {
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('あ'), 2);
        assert_eq!(char_width('한'), 2);
        assert_eq!(char_width('Ａ'), 2); // fullwidth latin
        assert_eq!(char_width('🚀'), 2);
    }

    #[test]
    fn latin_is_one_cell() {
        assert_eq!(char_width('é'), 1);
        assert_eq!(char_width('Ω'), 1);
    }
}
