//! VT/ECMA-48 byte-stream parser (hand-written, no external deps).
//!
//! States follow the ECMA-48 diagram: Ground, Escape, EscapeIntermediate,
//! CsiEntry, CsiParam, CsiIntermediate, CsiIgnore, OscString, DcsEntry,
//! DcsParam, DcsIntermediate, DcsPassthrough, DcsIgnore, SosPmApcString.
//!
//! The parser is push-driven: `Parser::feed()` walks bytes and invokes the
//! [`Perform`] implementation. The host (`term::Terminal`) implements the
//! actions; this module has no terminal state of its own.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    OscString,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    SosPmApcString,
}

/// One CSI/DCS parameter, with optional sub-parameters (`38:5:196`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub value: u16,
    pub sub: Vec<u16>,
}

impl Param {
    pub fn new(v: u16) -> Self {
        Param {
            value: v,
            sub: Vec::new(),
        }
    }
}

/// Actions a consumer must implement. Defaults are no-ops so tests can
/// implement only what they need.
#[allow(unused_variables)]
pub trait Perform {
    /// Printable character (already width-checked by the host).
    fn print(&mut self, ch: char) {}
    /// C0 control byte.
    fn execute(&mut self, byte: u8) {}
    /// CSI final byte with params/intermediates (e.g. `ESC [ 1 ; 31 m`).
    fn csi_dispatch(
        &mut self,
        params: &[Param],
        intermediates: &[u8],
        final_byte: u8,
        private: bool,
    ) {
    }
    /// ESC + final (e.g. `ESC 7`, `ESC ( B`).
    fn esc_dispatch(&mut self, intermediates: &[u8], final_byte: u8) {}
    /// OSC terminator reached with accumulated payload.
    fn osc_dispatch(&mut self, params: &[&[u8]]) {}
    /// DCS hook (enter passthrough), data, unhook.
    fn hook(&mut self, params: &[Param], intermediates: &[u8], final_byte: u8) {}
    fn put(&mut self, byte: u8) {}
    fn unhook(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct Parser {
    state: State,
    params: Vec<Param>,
    cur_param: Option<u16>,
    cur_sub: Option<Vec<u16>>,
    in_sub: bool,
    intermediates: Vec<u8>,
    osc: Vec<u8>,
    dcs: Vec<u8>,
    private: bool,
    /// Incomplete UTF-8 sequence carried over from the previous feed.
    pending_utf8: Vec<u8>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            state: State::Ground,
            params: Vec::new(),
            cur_param: None,
            cur_sub: None,
            in_sub: false,
            intermediates: Vec::new(),
            osc: Vec::new(),
            dcs: Vec::new(),
            private: false,
            pending_utf8: Vec::new(),
        }
    }

    /// Feed arbitrary bytes; UTF-8 sequences may be split across calls.
    pub fn feed<P: Perform>(&mut self, bytes: &[u8], performer: &mut P) {
        // Re-join a carried-over partial UTF-8 tail with the new bytes.
        let mut input: Vec<u8>;
        let slice: &[u8] = if self.pending_utf8.is_empty() {
            bytes
        } else {
            input = std::mem::take(&mut self.pending_utf8);
            input.extend_from_slice(bytes);
            &input
        };
        let bytes = slice;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if self.state == State::Ground && b >= 0x20 {
                // Decode UTF-8 printable runs quickly.
                if b < 0x80 {
                    self.perform_print(b as char, performer);
                    i += 1;
                    continue;
                }
                let len = utf8_len(b);
                if i + len > bytes.len() {
                    // Incomplete sequence at chunk end: carry it over.
                    self.pending_utf8.extend_from_slice(&bytes[i..]);
                    return;
                }
                if bytes[i + 1..i + len].iter().all(|&x| x & 0xC0 == 0x80) {
                    if let Ok(s) = std::str::from_utf8(&bytes[i..i + len]) {
                        if let Some(c) = s.chars().next() {
                            self.perform_print(c, performer);
                        }
                    }
                    i += len;
                    continue;
                }
                // Invalid sequence: skip the byte.
                i += 1;
                continue;
            }
            self.step(b, performer);
            i += 1;
        }
    }

    fn perform_print<P: Perform>(&mut self, ch: char, performer: &mut P) {
        performer.print(ch);
    }

    fn push_param(&mut self) {
        let v = self.cur_param.take().unwrap_or(0);
        if self.in_sub {
            let sub = self.cur_sub.get_or_insert_with(Vec::new);
            sub.push(v);
            let sub = self.cur_sub.take().unwrap_or_default();
            self.in_sub = false;
            if let Some(last) = self.params.last_mut() {
                last.sub = sub;
            } else {
                self.params.push(Param { value: 0, sub });
            }
        } else {
            self.params.push(Param::new(v));
        }
    }

    fn step<P: Perform>(&mut self, b: u8, performer: &mut P) {
        use State::*;
        // Common aborts; ESC is handled per-state because OSC/DCS/PMC consume
        // it as the first half of ST (ESC \).
        match b {
            0x18 | 0x1A => {
                self.state = Ground;
                return;
            }
            0x1B if !matches!(
                self.state,
                OscString | DcsPassthrough | SosPmApcString | DcsEntry | DcsParam | DcsIntermediate
            ) =>
            {
                self.reset_sequence();
                self.state = Escape;
                return;
            }
            _ => {}
        }
        match self.state {
            Ground => match b {
                0x07 => performer.execute(0x07), // BEL
                0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F | 0x84 | 0x85 | 0x90 => {
                    performer.execute(b)
                }
                0x00..=0x1F | 0x7F => { /* other C0 ignored in ground */ }
                _ => unreachable!("printables handled in feed"),
            },
            Escape => match b {
                b'[' => {
                    self.reset_sequence();
                    self.state = CsiEntry;
                }
                b']' => {
                    self.reset_sequence();
                    self.state = OscString;
                }
                b'P' => {
                    self.reset_sequence();
                    self.state = DcsEntry;
                }
                b'X' | b'^' | b'_' => {
                    self.reset_sequence();
                    self.state = SosPmApcString;
                }
                0x20..=0x2F => {
                    self.intermediates.push(b);
                    self.state = EscapeIntermediate;
                }
                0x30..=0x7E => {
                    let f = b;
                    self.state = Ground;
                    performer.esc_dispatch(&self.intermediates.clone(), f);
                }
                0x00..=0x1F => performer.execute(b),
                _ => self.state = Ground,
            },
            EscapeIntermediate => match b {
                0x20..=0x2F => self.intermediates.push(b),
                0x30..=0x7E => {
                    let f = b;
                    self.state = Ground;
                    performer.esc_dispatch(&self.intermediates.clone(), f);
                }
                _ => self.state = Ground,
            },
            CsiEntry => match b {
                b'?' | b'<' | b'=' | b'>' => {
                    if b == b'?' {
                        self.private = true;
                    }
                    self.state = CsiParam;
                }
                0x30..=0x39 => {
                    self.cur_param = Some((b - b'0') as u16);
                    self.state = CsiParam;
                }
                b':' => {
                    // First sub-param after a bare entry digit: move it in.
                    let v = self.cur_param.take().unwrap_or(0);
                    self.params.push(Param::new(v));
                    self.in_sub = true;
                    self.state = CsiParam;
                }
                b';' => {
                    self.push_param();
                    self.state = CsiParam;
                }
                0x20..=0x2F => {
                    self.intermediates.push(b);
                    self.state = CsiIntermediate;
                }
                0x40..=0x7E => {
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let priv_ = std::mem::take(&mut self.private);
                    self.state = Ground;
                    performer.csi_dispatch(&params, &self.intermediates.clone(), f, priv_);
                }
                _ => self.state = CsiIgnore,
            },
            CsiParam => match b {
                0x30..=0x39 => {
                    let v = self.cur_param.unwrap_or(0).min(15999);
                    self.cur_param = Some(v * 10 + (b - b'0') as u16);
                }
                b';' => self.push_param(),
                b':' => {
                    let v = self.cur_param.take().unwrap_or(0);
                    if self.in_sub {
                        self.cur_sub.get_or_insert_with(Vec::new).push(v);
                    } else {
                        self.params.push(Param::new(v));
                        self.in_sub = true;
                    }
                }
                0x20..=0x2F => {
                    self.push_param();
                    self.intermediates.push(b);
                    self.state = CsiIntermediate;
                }
                0x40..=0x7E => {
                    self.push_param();
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let priv_ = std::mem::take(&mut self.private);
                    self.state = Ground;
                    performer.csi_dispatch(&params, &self.intermediates.clone(), f, priv_);
                }
                _ => self.state = CsiIgnore,
            },
            CsiIntermediate => match b {
                0x20..=0x2F => self.intermediates.push(b),
                0x30..=0x3F => self.state = CsiIgnore,
                0x40..=0x7E => {
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let priv_ = std::mem::take(&mut self.private);
                    self.state = Ground;
                    performer.csi_dispatch(&params, &self.intermediates.clone(), f, priv_);
                }
                _ => self.state = Ground,
            },
            CsiIgnore => {
                if let 0x40..=0x7E = b {
                    self.reset_sequence();
                    self.state = Ground;
                }
            }
            OscString => {
                // ST is ESC \ (split across bytes): stash ESC, dispatch on \.
                if b == 0x1B {
                    self.osc.push(b);
                    return;
                }
                if b == 0x5C && self.osc.ends_with(&[0x1B]) {
                    self.osc.pop();
                    self.osc_dispatch_now(performer);
                    return;
                }
                if self.osc.ends_with(&[0x1B]) {
                    // A real escape sequence begins: abort OSC, reprocess byte.
                    self.osc.pop();
                    self.reset_sequence();
                    self.state = Escape;
                    self.step(b, performer);
                    return;
                }
                match b {
                    0x07 | 0x9C => self.osc_dispatch_now(performer),
                    0x00..=0x1F => {
                        self.reset_sequence();
                        self.state = Ground;
                    }
                    _ => self.osc.push(b),
                }
            }
            DcsEntry => match b {
                0x30..=0x39 => {
                    self.cur_param = Some((b - b'0') as u16);
                    self.state = DcsParam;
                }
                b':' => {
                    let v = self.cur_param.take().unwrap_or(0);
                    self.params.push(Param::new(v));
                    self.in_sub = true;
                    self.state = DcsParam;
                }
                b';' => {
                    self.push_param();
                    self.state = DcsParam;
                }
                b'?' | b'<' | b'=' | b'>' => self.state = DcsParam,
                0x20..=0x2F => {
                    self.intermediates.push(b);
                    self.state = DcsIntermediate;
                }
                0x40..=0x7E => {
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let _priv_ = std::mem::take(&mut self.private);
                    self.state = DcsPassthrough;
                    performer.hook(&params, &self.intermediates.clone(), f);
                }
                _ => self.state = DcsIgnore,
            },
            DcsParam => match b {
                0x30..=0x39 => {
                    let v = self.cur_param.unwrap_or(0).min(15999);
                    self.cur_param = Some(v * 10 + (b - b'0') as u16);
                }
                b';' => self.push_param(),
                b':' => {
                    let v = self.cur_param.take().unwrap_or(0);
                    if self.in_sub {
                        self.cur_sub.get_or_insert_with(Vec::new).push(v);
                    } else {
                        self.params.push(Param::new(v));
                        self.in_sub = true;
                    }
                }
                0x20..=0x2F => {
                    self.push_param();
                    self.intermediates.push(b);
                    self.state = DcsIntermediate;
                }
                0x40..=0x7E => {
                    self.push_param();
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let _priv_ = std::mem::take(&mut self.private);
                    self.state = DcsPassthrough;
                    performer.hook(&params, &self.intermediates.clone(), f);
                }
                _ => self.state = DcsIgnore,
            },
            DcsIntermediate => match b {
                0x20..=0x2F => self.intermediates.push(b),
                0x40..=0x7E => {
                    let f = b;
                    let params = std::mem::take(&mut self.params);
                    let _priv_ = std::mem::take(&mut self.private);
                    self.state = DcsPassthrough;
                    performer.hook(&params, &self.intermediates.clone(), f);
                }
                _ => self.state = DcsIgnore,
            },
            DcsPassthrough | SosPmApcString | DcsIgnore => {
                let passthrough = self.state == DcsPassthrough;
                if b == 0x1B {
                    self.dcs.push(b);
                    return;
                }
                if b == 0x5C && self.dcs.ends_with(&[0x1B]) {
                    self.dcs.pop();
                    if passthrough {
                        performer.unhook();
                    }
                    self.reset_sequence();
                    self.state = Ground;
                    return;
                }
                if self.dcs.ends_with(&[0x1B]) {
                    self.dcs.pop();
                    if passthrough {
                        performer.unhook();
                    }
                    self.reset_sequence();
                    self.state = Escape;
                    self.step(b, performer);
                    return;
                }
                match b {
                    0x9C => {
                        if passthrough {
                            performer.unhook();
                        }
                        self.reset_sequence();
                        self.state = Ground;
                    }
                    0x07 if passthrough => {
                        performer.unhook();
                        self.reset_sequence();
                        self.state = Ground;
                    }
                    _ if b >= 0x20 && passthrough => {
                        performer.put(b);
                        self.dcs.push(b);
                    }
                    _ => {}
                }
            }
        }
    }

    fn osc_dispatch_now<P: Perform>(&mut self, performer: &mut P) {
        let data = std::mem::take(&mut self.osc);
        let params: Vec<&[u8]> = data.split(|&b| b == b';').collect();
        performer.osc_dispatch(&params);
        self.reset_sequence();
        self.state = State::Ground;
    }

    fn reset_sequence(&mut self) {
        self.params.clear();
        self.cur_param = None;
        self.cur_sub = None;
        self.in_sub = false;
        self.intermediates.clear();
        self.osc.clear();
        self.dcs.clear();
        self.private = false;
    }

    pub fn state_name(&self) -> &'static str {
        match self.state {
            State::Ground => "ground",
            State::Escape => "escape",
            _ => "sequence",
        }
    }
}

fn utf8_len(first: u8) -> usize {
    if first & 0xE0 == 0xC0 {
        2
    } else if first & 0xF0 == 0xE0 {
        3
    } else if first & 0xF8 == 0xF0 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    type OscLog = Vec<Vec<Vec<u8>>>;
    type CsiLog = Vec<(Vec<Param>, Vec<u8>, u8, bool)>;

    #[derive(Default)]
    struct Rec {
        printed: RefCell<String>,
        executes: RefCell<Vec<u8>>,
        csi: RefCell<CsiLog>,
        esc: RefCell<Vec<(Vec<u8>, u8)>>,
        osc: RefCell<OscLog>,
        dcs_data: RefCell<Vec<u8>>,
        hooked: RefCell<bool>,
        unhooks: RefCell<u32>,
    }

    impl Perform for Rec {
        fn print(&mut self, ch: char) {
            self.printed.borrow_mut().push(ch);
        }
        fn execute(&mut self, b: u8) {
            self.executes.borrow_mut().push(b);
        }
        fn csi_dispatch(&mut self, params: &[Param], inter: &[u8], f: u8, priv_: bool) {
            self.csi
                .borrow_mut()
                .push((params.to_vec(), inter.to_vec(), f, priv_));
        }
        fn esc_dispatch(&mut self, inter: &[u8], f: u8) {
            self.esc.borrow_mut().push((inter.to_vec(), f));
        }
        fn osc_dispatch(&mut self, params: &[&[u8]]) {
            self.osc
                .borrow_mut()
                .push(params.iter().map(|p| p.to_vec()).collect());
        }
        fn hook(&mut self, _p: &[Param], _i: &[u8], _f: u8) {
            *self.hooked.borrow_mut() = true;
        }
        fn put(&mut self, b: u8) {
            self.dcs_data.borrow_mut().push(b);
        }
        fn unhook(&mut self) {
            *self.unhooks.borrow_mut() += 1;
        }
    }

    fn feed(s: &str) -> Rec {
        let mut p = Parser::new();
        let mut r = Rec::default();
        p.feed(s.as_bytes(), &mut r);
        r
    }

    #[test]
    fn plain_text_prints() {
        let r = feed("hello 世界");
        assert_eq!(*r.printed.borrow(), "hello 世界".to_string());
        assert!(r.csi.borrow().is_empty());
    }

    #[test]
    fn utf8_split_across_feeds() {
        let mut p = Parser::new();
        let mut r = Rec::default();
        let bytes = "世".as_bytes();
        p.feed(&bytes[..1], &mut r);
        p.feed(&bytes[1..], &mut r);
        assert_eq!(*r.printed.borrow(), "世".to_string());
    }

    #[test]
    fn csi_basic_params() {
        let r = feed("\x1b[10;20H");
        let csi = r.csi.borrow();
        assert_eq!(csi.len(), 1);
        assert_eq!(csi[0].2, b'H');
        assert_eq!(csi[0].0, vec![Param::new(10), Param::new(20)]);
    }

    #[test]
    fn csi_missing_params_default_zero() {
        let r = feed("\x1b[H");
        let csi = r.csi.borrow();
        assert_eq!(
            csi[0].0,
            Vec::<Param>::new(),
            "no params; consumer applies defaults"
        );
    }

    #[test]
    fn csi_private_marker() {
        let r = feed("\x1b[?25l");
        let csi = r.csi.borrow();
        assert!(csi[0].3, "private '?' flag");
        assert_eq!(csi[0].2, b'l');
    }

    #[test]
    fn csi_colon_subparams() {
        let r = feed("\x1b[38:5:196m");
        let csi = r.csi.borrow();
        let p = &csi[0].0;
        assert_eq!(p[0].value, 38);
        assert_eq!(p[0].sub, vec![5, 196]);
    }

    #[test]
    fn csi_semicolon_color_forms() {
        let r = feed("\x1b[38;5;196;1m");
        let csi = r.csi.borrow();
        let vals: Vec<u16> = csi[0].0.iter().map(|p| p.value).collect();
        assert_eq!(vals, vec![38, 5, 196, 1]);
    }

    #[test]
    fn csi_intermediate_marker() {
        let r = feed("\x1b[ q"); // DECSCUSR with space intermediate
        let csi = r.csi.borrow();
        assert_eq!(csi[0].1, b" ".to_vec());
        assert_eq!(csi[0].2, b'q');
    }

    #[test]
    fn csi_invalid_enters_ignore() {
        let r = feed("\x1b[<Xhello");
        assert_eq!(
            *r.printed.borrow(),
            "hello".to_string(),
            "garbage swallowed"
        );
    }

    #[test]
    fn esc_sequences() {
        let r = feed("\x1b7\x1b8\x1bM\x1bc");
        let esc = r.esc.borrow();
        let finals: Vec<u8> = esc.iter().map(|e| e.1).collect();
        assert_eq!(finals, vec![b'7', b'8', b'M', b'c']);
    }

    #[test]
    fn esc_charset_designation() {
        let r = feed("\x1b(0\x1b)B");
        let esc = r.esc.borrow();
        assert_eq!(esc[0].0, b"(".to_vec());
        assert_eq!(esc[0].1, b'0');
        assert_eq!(esc[1].0, b")".to_vec());
        assert_eq!(esc[1].1, b'B');
    }

    #[test]
    fn osc_title_and_bel() {
        let r = feed("\x1b]2;my title\x07tail");
        let osc = r.osc.borrow();
        assert_eq!(osc.len(), 1);
        assert_eq!(osc[0][0], b"2".to_vec());
        assert_eq!(osc[0][1], b"my title".to_vec());
        assert_eq!(*r.printed.borrow(), "tail".to_string());
    }

    #[test]
    fn osc_st_terminator() {
        let r = feed("\x1b]0;abc\x1b\\next");
        let osc = r.osc.borrow();
        assert_eq!(osc.len(), 1);
        assert_eq!(osc[0][1], b"abc".to_vec());
        assert_eq!(*r.printed.borrow(), "next".to_string());
    }

    #[test]
    fn osc_multiple_semicolons() {
        let r = feed("\x1b]52;c;aGVsbG8=\x07");
        let osc = r.osc.borrow();
        assert_eq!(osc[0].len(), 3);
        assert_eq!(osc[0][2], b"aGVsbG8=".to_vec());
    }

    #[test]
    fn can_and_sub_abort_sequences() {
        let mut p = Parser::new();
        let mut r = Rec::default();
        p.feed(b"\x1b[3\x18rest", &mut r);
        assert_eq!(*r.printed.borrow(), "rest".to_string());
    }

    #[test]
    fn control_chars_execute() {
        let r = feed("a\rb\nc\x07");
        let ex = r.executes.borrow();
        assert!(ex.contains(&b'\r') && ex.contains(&b'\n') && ex.contains(&0x07));
    }

    #[test]
    fn dcs_passthrough_collects() {
        // DCS 0 ! r |1/2 ST: hook(final='r'), data "|1/2".
        let r = feed("\x1bP0!r|1/2\x1b\\");
        assert!(*r.hooked.borrow());
        assert_eq!(*r.dcs_data.borrow(), b"|1/2".to_vec());
        assert_eq!(*r.unhooks.borrow(), 1);
    }

    #[test]
    fn parser_resets_between_sequences() {
        let mut p = Parser::new();
        let mut r = Rec::default();
        p.feed(b"\x1b[5A\x1b[3B", &mut r);
        let csi = r.csi.borrow();
        assert_eq!(csi[0].0, vec![Param::new(5)]);
        assert_eq!(
            csi[1].0,
            vec![Param::new(3)],
            "params must not leak across sequences"
        );
    }

    #[test]
    fn long_input_no_state_leak() {
        let mut p = Parser::new();
        let mut r = Rec::default();
        let mut s = String::new();
        for _ in 0..200 {
            s.push_str("\x1b[31mX\x1b[0m");
        }
        p.feed(s.as_bytes(), &mut r);
        assert_eq!(r.csi.borrow().len(), 400);
        assert_eq!(*r.printed.borrow(), "X".repeat(200));
    }

    #[test]
    fn osc_ignores_raw_control_chars_inside() {
        // C0 other than BEL/ST aborts OSC.
        let r = feed("\x1b]2;ab\ncd\x07Z");
        let osc = r.osc.borrow();
        assert!(osc.is_empty(), "LF aborted the OSC");
        assert_eq!(*r.printed.borrow(), "cdZ");
    }
}
