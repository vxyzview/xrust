# Release notes

## Unreleased

### Responsive UI/UX for all window sizes
- Window-width classes (Material 3 breakpoints, manual — no new
  dependencies): Compact <600dp, Medium 600–839dp, Expanded ≥840dp.
- Adaptive navigation: bottom bar on compact phones, side navigation rail
  on tablets / landscape / split screen; terminal and installer stay
  fullscreen on every shape.
- Terminal: extra-keys rail stands vertically in landscape (grid keeps full
  height), key chips now ≥44dp touch targets, `imePadding` keeps the grid
  above the soft keyboard.
- All 12 content screens center and cap their width (480–640dp) on wide
  windows so lists and text do not stretch edge to edge.
- Scrollable added where fixed stacks overflowed (Font, Keyboard, Profile
  editor, Installer); file rows enforce 48dp minimum height; session action
  row scrolls instead of clipping at 200% font scale.

## v1.0.0 (initial release)

### Highlights
- xrust — a Rust-first terminal for Android. VT/ECMA-48 parser, wide-char
  grid, scrollback with true reflow, xterm key encoding, PTY sessions.
  134 tests green, clippy `-D warnings` clean.
- Alpine 3.24.4 userland via PRoot 5.5.0, installed with SHA-256
  verification and traversal-safe extraction.
- Compose Material 3 UI with 12 screens, AMOLED pure black + Rose Ash
  themes, 12 terminal color schemes, 25 terminal parameters.
- Dual-pane file manager (Android ↔ rootfs), profile system with JSON
  import/export.

### Known limitations
- Host shell sessions are restricted by the Android sandbox (see
  docs/HOST_SHELL_LIMITS.md).
- PRoot is not a security boundary.
- Scrollback re-flow covers history + screen content; cursor column is
  best-effort on resize.
