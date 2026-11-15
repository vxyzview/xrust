//! xrust-core — the Rust heart of xrust.
//!
//! Everything that must be *correct* lives here: the VT/ECMA-48 parser,
//! the screen grid model, scrollback, reflow on resize, selection, keyboard
//! encoding, and the PTY/session layer. The Kotlin/Compose side only renders
//! snapshots and forwards input; it owns no terminal logic.
//!
//! Module map:
//! - [`vte`]: byte-stream state machine producing [`vte::Perform`] callbacks
//! - [`grid`]: cell grid, wide-char handling, line ops, tab stops
//! - [`scrollback`]: bounded history of scrolled-out lines (with wrap flags)
//! - [`reflow`]: history re-wrap on width change
//! - [`selection`]: anchor/extent selection with copy semantics
//! - [`keymap`]: key events → PTY byte sequences (xterm-compatible)
//! - [`color`]: color type + built-in schemes (incl. AMOLED pure black)
//! - [`term`]: the `Terminal` actor implementing `Perform`; owns modes
//! - [`pty`]: openpty/fork/exec, child reaping, process-group signals
//! - [`android_jni`]: JNI bridge for the Android app (cdylib exports)

pub mod android_jni;
pub mod color;
pub mod grid;
pub mod keymap;
pub mod pty;
pub mod reflow;
pub mod scrollback;
pub mod selection;
pub mod term;
pub mod vte;
pub mod wcwidth;

pub use grid::{Cell, Grid};
pub use scrollback::Scrollback;
pub use term::{ModeFlags, Terminal};
