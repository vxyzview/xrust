# Product

<!-- impeccable:product-schema 1 -->

## Platform

android

## Users

Three audiences, roughly equally weighted — no single dominant one:

- **Developers and sysadmins** who need a real Linux userland on their
  Android phone or tablet: quick scripts, pipelines, on-call fixes,
  file inspection.
- **Students and learners** exploring Linux and the command line in a
  safe, self-contained playground.
- **Power users / device tinkerers** who want rootless Linux on the
  device without a coding background.

Confirmed via interview (2026-11-15).

## Product Purpose

xrust is a terminal app for Android that ships a complete Alpine Linux
userland, sandboxed with PRoot and requiring no root. It exists so an
Android device can run a real shell (`/bin/sh`, `apk`, the full 28 671
package Alpine repository) safely and correctly. Success means the
terminal behaves identically and correctly on every device — same
parser, same grid, same key encoding — and that install is safe by
construction (checksum-verified download, traversal/symlink-proof
extraction).

## Positioning

The claim a competing Android terminal could not truthfully copy:
**a Rust-first terminal — hand-written VT/ECMA-48 parser, wide-char
grid, xterm key encoder, PTY management, 135 tests — paired with a
full, SHA-256-verified Alpine userland that needs no root.** Correctness
via the Rust core plus the safe rootless sandbox is the differentiator;
protect it. Confirmed via interview.

## Operating Context

- Runs entirely on-device, offline after the initial Alpine download
  (`alpine-minirootfs-3.24.4`, verified against `dl-cdn.alpinelinux.org`).
- Used where a desktop is unavailable or inconvenient: commuting,
  travel, quick interventions; also as a learning environment.
- Distribution: signed APKs auto-published on `v*` tags via GitHub
  Releases; Android 8.0+ (side-load, unknown sources allowed).
- Host-shell sessions (Android's own `/system/bin/sh`, no PRoot) are
  intentionally restricted by the OS sandbox — documented as a feature
  limit in `docs/HOST_SHELL_LIMITS.md`.

## Capabilities and Constraints

- 12 screens: sessions, interactive terminal, installer, dual-pane file
  manager (Android ↔ rootfs), settings hub + 25 tunable terminal
  parameters, display, font, 12 terminal color schemes, keyboard,
  profiles + profile editor, about.
- Dynamic Material You color on Android 12+; 4 complete-theme modes
  (including AMOLED pure black and Rose Ash).
- Rust core (`xrust-core`) owns everything that must be correct:
  parsing, state, I/O sequencing. Kotlin only renders snapshots and
  forwards input.
- PRoot 5.5.0 built per-ABI in CI; installer verifies SHA-256 before
  extraction and rejects path traversal and absolute symlinks.
- JNI boundary returns plain booleans; no exceptions across the border.
- Architecture doc: `docs/ARCHITECTURE.md`; e2e suite
  `scripts/e2e.sh` runs the full stack on Linux (PTY → parser → grid →
  PRoot → real Alpine shell).

## Brand Commitments

Binding per interview (2026-11-15):

- **Name:** "xrust" (lowercase) and the existing visual identity assets
  (`docs/assets/banner.svg`).
- **License:** MIT for this project; PRoot (GPL-2.0) ships as a separate
  executable.
- **Design language:** Jetpack Compose Material You 3 (Material
  Expressive pass in 1.3.0) is the app's design system; dynamic color
  behavior is part of the product.
- **Rendering rule:** no WebView anywhere — terminal rendering is a
  Compose `Canvas` over the native snapshot. This is an architectural
  commitment, not a preference.

## Evidence on Hand

- Real, verified upstream artifact: the official Alpine 3.24.4
  minirootfs tarball, SHA-256-checked in the e2e suite
  (`E2E_ALL_GREEN`, 28 671 packages confirmed via real `apk update`).
- 135 Rust unit + integration tests; clippy `-D warnings` clean;
  rustfmt clean; CI badge on README.
- README, `docs/ARCHITECTURE.md`, `docs/HOST_SHELL_LIMITS.md`,
  release notes and checklist under `docs/`.
- **Absences future work must not fabricate:** no user research, no
  testimonials, no press coverage, no adoption/usage metrics, no
  benchmark comparisons against other Android terminals.

## Product Principles

1. **Correctness lives in the core.** Anything that must be right
   (parse, sequence, store, encode) belongs in Rust and under test;
   the Kotlin layer renders and forwards only.
2. **Safety is structural, not promised.** Checksums before extract,
   traversal rejected by construction, sandbox without root — the safe
   path is the only path.
3. **Identical behavior everywhere.** The terminal must feel the same
   on any device, any Android version the app supports.
4. **Native all the way down.** Compose Canvas, no WebView, no web
   technology in the render path.
5. **Three audiences, one surface.** Features must stay legible to a
   learner and precise enough for a sysadmin without splitting the app
   into two products.

## Accessibility & Inclusion

No product-specific accessibility requirement was established in the
interview. (TalkBack support was offered as a binding commitment and
not selected.) Standard Android platform behavior applies as default.
