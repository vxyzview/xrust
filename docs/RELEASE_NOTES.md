# Release notes

## v1.3.0 — Material You expressive pass

### Shape system (app-wide)
- New app shape vocabulary, rounder than M3 defaults: text fields/chips 10dp,
  cards 18dp, FABs 24dp, dialogs and sheets 28dp. Every component across all
  12 screens inherits it from the theme.

### Typography scale
- Full explicit type scale (display 30sp → label 11sp) with real line
  heights; list titles tightened to M3's 16sp with semibold weight, so
  hierarchy reads consistently instead of per-screen.
- Monospace carries the terminal vernacular in small doses: status lines,
  metadata labels, chips.

### Motion
- Install progress bar glides between percentages (animated float) instead
  of stepping; state changes on the installer page and the home status card
  resize smoothly (animateContentSize) instead of jumping.
- Screen transitions (fade + slide, ~200ms) were already in place and stay.

### Home screen hero
- The Alpine status card is now the one expressive element: the status
  reads as terminal output (`$ xrust status` → `alpine 3.24.4 ready`) on a
  primary container, with "Open shell" as a tonal secondary action. The
  rest of the home screen stays quiet Material You.

## v1.2.0 — Material You, and the fixes that matter

### Fixed
- **SHA-256 verification no longer reports a false mismatch.** Upstream
  checksum files are `<hash>  <filename>` on one line; the verifier now
  compares only the hash token, so installing Alpine succeeds instead of
  failing with "got X, want X" (where X was already equal).
- **"Open" no longer force-closes.** The terminal route was hardcoded to
  the Alpine profile even for the host shell card; with no rootfs installed
  the PRoot launcher threw and killed the process. Routing is now
  profile-aware (`terminal/{profileId}`): the host shell opens
  `host-default` directly and needs no rootfs, and each card opens its own
  profile.
- **Crash hardening in the controller**: `nativeSnapshot` nullability
  coalesced at both call sites.

### Material You 3, everywhere
- Every app theme (Alpine Dark / Alpine Light / AMOLED Pure Black / Rose
  Ash) now carries a **complete** Material 3 `ColorScheme` — all ~30 tonal
  roles set explicitly — so no widget falls back to the framework's
  light-scheme defaults (the source of off-palette pink/blue one-off
  widgets in v1.1.x).
- SYSTEM theme uses **dynamic wallpaper-derived color** on Android 12+
  (dynamicDark/LightColorScheme), falling back to Alpine themes below.
- Installer errors render in a proper M3 error-container card instead of
  bare error text.

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

## v1.1.0 — smooth, faster, fluid

### Input latency & concurrency (Rust core)
- `nativePoll` now sleeps on a per-session condvar that the reader thread
  signals after every feed: keystroke echo and program output reach the UI
  within microseconds, instead of up to one 20ms sleep quantum later, with
  no busy mutex churn while idle.
- Keystroke / paste writes copy the pty fd under the mutex and write
  without it — a paste into a flooded pipe can no longer freeze concurrent
  typing for up to 200ms per EAGAIN retry.
- `mem::take` parser swap in `Term::feed` (clippy 1.100 clean).

### Terminal rendering (Compose)
- The draw loop reuses one `Paint` and one `StringBuilder`: the old loop
  allocated a fresh Paint per visible cell per frame (~1000 objects/frame,
  GC jank during scroll floods).
- Same-style character runs are batched into a single `drawText` call,
  cutting JNI text calls roughly an order of magnitude on ordinary output.

### App fluidity (Compose)
- Navigation transitions switched from the sluggish ~700ms default fade to
  snappy ~200ms fade + slide (forward and pop).
- Poll loop throttles to 8ms between refreshes under output floods, so a
  `cat` of a huge file cannot spin the dispatcher, while staying above
  display cadence; redundant dispatcher hop removed.
- LazyList `contentType` hints on Files / Profiles / Color schemes rows
  improve item reuse while scrolling.

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
