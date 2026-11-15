# Release notes

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
