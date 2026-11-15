//! Keyboard events → PTY byte sequences, xterm-compatible.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    F(u8),
}

pub const MOD_SHIFT: u8 = 1;
pub const MOD_ALT: u8 = 2;
pub const MOD_CTRL: u8 = 4;

/// Modifier parameter for CSI 1;mX / CSI n;m~ forms (xterm convention).
fn mod_param(mods: u8) -> u8 {
    1 + mods
}

/// Encode a key press. `app_cursor` reflects DECCKM (?1), `app_keypad` the
/// application keypad mode (DECPAM).
pub fn encode(key: Key, mods: u8, app_cursor: bool, _app_keypad: bool) -> Vec<u8> {
    let m = mod_param(mods);
    let alt = mods & MOD_ALT != 0;
    let mut out = Vec::with_capacity(8);
    // CSI-encoded keys carry the modifier inside the sequence itself; no
    // ESC prefix (unlike legacy Char/Enter/F1-4 SS3 forms).
    let csi = |out: &mut Vec<u8>, body: String| {
        out.extend_from_slice(body.as_bytes());
    };

    match key {
        Key::Char(c) => {
            if mods & MOD_CTRL != 0 {
                let up = c.to_ascii_uppercase();
                let code: u8 = match up {
                    'A'..='Z' => (up as u8) & 0x1F,
                    ' ' | '@' => 0x00,
                    '[' => 0x1B,
                    '\\' => 0x1C,
                    ']' => 0x1D,
                    '^' => 0x1E,
                    '_' => 0x1F,
                    '?' => 0x7F,
                    _ => return out, // unmapped control combo
                };
                if alt {
                    out.push(0x1B);
                }
                out.push(code);
                return out;
            }
            if alt {
                out.push(0x1B);
            }
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            out
        }
        Key::Enter => {
            if alt {
                out.push(0x1B);
            }
            out.push(b'\r');
            out
        }
        Key::Backspace => {
            if alt {
                out.push(0x1B);
            }
            out.push(if mods & MOD_CTRL != 0 { 0x08 } else { 0x7F });
            out
        }
        Key::Tab => {
            if mods & MOD_SHIFT != 0 {
                csi(&mut out, "\x1b[Z".into());
            } else if alt || mods & MOD_CTRL != 0 {
                csi(&mut out, "\x1b[I".into());
            } else {
                out.push(b'\t');
            }
            out
        }
        Key::Escape => {
            out.push(0x1B);
            out
        }
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            let f = match key {
                Key::ArrowUp => 'A',
                Key::ArrowDown => 'B',
                Key::ArrowRight => 'C',
                Key::ArrowLeft => 'D',
                _ => unreachable!(),
            };
            if mods == 0 {
                if alt {
                    out.push(0x1B);
                }
                if app_cursor {
                    out.extend_from_slice(format!("\x1bO{f}").as_bytes());
                } else {
                    out.extend_from_slice(format!("\x1b[{f}").as_bytes());
                }
            } else {
                csi(&mut out, format!("\x1b[1;{m}{f}"));
            }
            out
        }
        Key::Home | Key::End => {
            let f = if key == Key::Home { 'H' } else { 'F' };
            if mods == 0 {
                if alt {
                    out.push(0x1B);
                }
                if app_cursor {
                    out.extend_from_slice(format!("\x1bO{f}").as_bytes());
                } else {
                    out.extend_from_slice(format!("\x1b[{f}").as_bytes());
                }
            } else {
                csi(&mut out, format!("\x1b[1;{m}{f}"));
            }
            out
        }
        Key::PageUp | Key::PageDown | Key::Insert | Key::Delete => {
            let n = match key {
                Key::Insert => 2,
                Key::Delete => 3,
                Key::PageUp => 5,
                _ => 6,
            };
            if mods == 0 {
                csi(&mut out, format!("\x1b[{n}~"));
            } else {
                csi(&mut out, format!("\x1b[{n};{m}~"));
            }
            out
        }
        Key::F(k) => match (k, mods) {
            (1..=4, 0) => {
                if alt {
                    out.push(0x1B);
                }
                let f = b"PQRS"[k as usize - 1];
                out.push(0x1B);
                out.push(b'O');
                out.push(f);
                out
            }
            (1..=4, _) => {
                let f = b"PQRS"[k as usize - 1] as char;
                csi(&mut out, format!("\x1b[1;{m}{f}"));
                out
            }
            (5..=12, _) => {
                let n = match k {
                    5 => 15,
                    6 => 17,
                    7 => 18,
                    8 => 19,
                    9 => 20,
                    10 => 21,
                    11 => 23,
                    _ => 24,
                };
                if mods == 0 {
                    csi(&mut out, format!("\x1b[{n}~"));
                } else {
                    csi(&mut out, format!("\x1b[{n};{m}~"));
                }
                out
            }
            _ => out,
        },
    }
}

/// Encode a scroll event as xterm mouse reporting (SGR mode or legacy).
pub fn encode_mouse(
    btn: u8,
    pressed: bool,
    col: usize,
    row: usize,
    sgr: bool,
    motion: bool,
) -> Vec<u8> {
    let code = if motion {
        32 + btn
    } else {
        (if pressed { 0 } else { 3 }) + btn
    };
    if sgr {
        let m = if motion || pressed { 'M' } else { 'm' };
        format!("\x1b[<{code};{};{}{m}", col + 1, row + 1).into_bytes()
    } else {
        let mut v = vec![0x1B, b'[', b'M', 32 + code];
        v.push(33 + col.min(94) as u8);
        v.push(33 + row.min(94) as u8);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(k: Key) -> String {
        String::from_utf8(encode(k, 0, false, false)).unwrap()
    }

    #[test]
    fn plain_chars() {
        assert_eq!(s(Key::Char('a')), "a");
        assert_eq!(s(Key::Char('Z')), "Z");
        assert_eq!(s(Key::Char(' ')), " ");
        assert_eq!(
            String::from_utf8(encode(Key::Char('中'), 0, false, false)).unwrap(),
            "中"
        );
    }

    #[test]
    fn enter_bs_tab_esc() {
        assert_eq!(s(Key::Enter), "\r");
        assert_eq!(s(Key::Backspace), "\x7f");
        assert_eq!(s(Key::Tab), "\t");
        assert_eq!(s(Key::Escape), "\x1b");
    }

    #[test]
    fn ctrl_combos() {
        assert_eq!(
            String::from_utf8(encode(Key::Char('c'), MOD_CTRL, false, false)).unwrap(),
            "\x03"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Char('d'), MOD_CTRL, false, false)).unwrap(),
            "\x04"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Char(' '), MOD_CTRL, false, false)).unwrap(),
            "\x00"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Char('['), MOD_CTRL, false, false)).unwrap(),
            "\x1b"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Char('u'), MOD_CTRL, false, false)).unwrap(),
            "\x15"
        );
    }

    #[test]
    fn alt_prefixes_esc() {
        assert_eq!(
            String::from_utf8(encode(Key::Char('x'), MOD_ALT, false, false)).unwrap(),
            "\x1bx"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Char('x'), MOD_ALT | MOD_CTRL, false, false)).unwrap(),
            "\x1b\x18"
        );
        assert_eq!(
            String::from_utf8(encode(Key::Enter, MOD_ALT, false, false)).unwrap(),
            "\x1b\r"
        );
    }

    #[test]
    fn arrows_normal_and_application_mode() {
        assert_eq!(s(Key::ArrowUp), "\x1b[A");
        assert_eq!(s(Key::ArrowDown), "\x1b[B");
        assert_eq!(s(Key::ArrowRight), "\x1b[C");
        assert_eq!(s(Key::ArrowLeft), "\x1b[D");
        let app = String::from_utf8(encode(Key::ArrowUp, 0, true, false)).unwrap();
        assert_eq!(app, "\x1bOA");
    }

    #[test]
    fn arrows_with_modifiers() {
        // Shift+Right → CSI 1;2C
        assert_eq!(
            String::from_utf8(encode(Key::ArrowRight, MOD_SHIFT, false, false)).unwrap(),
            "\x1b[1;2C"
        );
        // Ctrl+Alt+Left → 1;7D (xterm modifier: 1 + shift1 + alt2 + ctrl4)
        assert_eq!(
            String::from_utf8(encode(Key::ArrowLeft, MOD_CTRL | MOD_ALT, false, false)).unwrap(),
            "\x1b[1;7D"
        );
        // Mods override app mode (xterm behavior)
        assert_eq!(
            String::from_utf8(encode(Key::ArrowUp, MOD_SHIFT, true, false)).unwrap(),
            "\x1b[1;2A"
        );
    }

    #[test]
    fn home_end_pages() {
        assert_eq!(s(Key::Home), "\x1b[H");
        assert_eq!(s(Key::End), "\x1b[F");
        assert_eq!(
            String::from_utf8(encode(Key::Home, 0, true, false)).unwrap(),
            "\x1bOH"
        );
        assert_eq!(s(Key::PageUp), "\x1b[5~");
        assert_eq!(s(Key::PageDown), "\x1b[6~");
        assert_eq!(s(Key::Insert), "\x1b[2~");
        assert_eq!(s(Key::Delete), "\x1b[3~");
        assert_eq!(
            String::from_utf8(encode(Key::Delete, MOD_CTRL, false, false)).unwrap(),
            "\x1b[3;5~"
        );
    }

    #[test]
    fn function_keys() {
        assert_eq!(s(Key::F(1)), "\x1bOP");
        assert_eq!(s(Key::F(4)), "\x1bOS");
        assert_eq!(s(Key::F(5)), "\x1b[15~");
        assert_eq!(s(Key::F(12)), "\x1b[24~");
        assert_eq!(
            String::from_utf8(encode(Key::F(2), MOD_SHIFT, false, false)).unwrap(),
            "\x1b[1;2Q"
        );
        assert_eq!(
            String::from_utf8(encode(Key::F(5), MOD_CTRL, false, false)).unwrap(),
            "\x1b[15;5~"
        );
    }

    #[test]
    fn shift_tab_backtab() {
        assert_eq!(
            String::from_utf8(encode(Key::Tab, MOD_SHIFT, false, false)).unwrap(),
            "\x1b[Z"
        );
    }

    #[test]
    fn mouse_sgr_and_legacy() {
        let sgr = String::from_utf8(encode_mouse(0, true, 10, 5, true, false)).unwrap();
        assert_eq!(sgr, "\x1b[<0;11;6M");
        let rel = String::from_utf8(encode_mouse(0, false, 10, 5, true, false)).unwrap();
        assert_eq!(rel, "\x1b[<3;11;6m");
        let legacy = encode_mouse(0, true, 0, 0, false, false);
        assert_eq!(legacy, vec![0x1B, b'[', b'M', 32, 33, 33]);
    }

    #[test]
    fn motion_encoding() {
        let mv = String::from_utf8(encode_mouse(0, false, 3, 4, true, true)).unwrap();
        assert_eq!(mv, "\x1b[<32;4;5M");
    }
}
