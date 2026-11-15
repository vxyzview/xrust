# Architecture

## Principles

1. **Rust-first**: everything that must be *correct* — parsing, state, I/O
   sequencing — lives in `xrust-core`. The Kotlin layer only renders
   snapshots and forwards input. This keeps the terminal behavior identical
   across devices and testable on any Linux host.
2. **No WebView** anywhere; rendering is a Compose `Canvas` over the native
   snapshot.
3. **Fail loudly**: installer verifies checksums before extraction; the
   extractor rejects path traversal and absolute symlinks; JNI calls return
   plain booleans instead of throwing across the boundary.

## Layer map

```
┌─ app (Kotlin, Compose M3) ──────────────────────────────────────┐
│ 12 screens · SettingsRepository (DataStore, 25 params)          │
│ ProfilesStore (JSON) · SessionsStore · FileManager              │
│ AlpineInstaller (download+SHA256+safe extract)                  │
│ ProotLauncher (argv assembly) · HostShell (restricted)          │
└───────────────▲─────────────────────────────┬───────────────────┘
        JNI (snapshot bytes, input bytes)      │ argv
┌───────────────┴─────────────────────────────▼───────────────────┐
│ xrust-core (Rust cdylib libxrust_core.so)                       │
│ vte parser → Terminal (modes, SGR, responses) → Grid (+scroll-  │
│ back w/ wrap flags) → encode_snapshot()                         │
│ keymap (xterm sequences) ← input                                │
│ pty: openpty/fork/exec, poll-based reader, reaping, SIGWINCH    │
│ reader thread: fd-level poll (NO lock held — see note)          │
└───────────────▲─────────────────────────────┬───────────────────┘
                │ master fd                   │ fork/exec
┌───────────────┴─────────────────────────────▼───────────────────┐
│ PRoot v5.5.0 → Alpine 3.24.4 rootfs (/bin/sh, apk, …)           │
└──────────────────────────────────────────────────────────────────┘
```

## Threading model

- One **reader thread per session** inside the Rust library: `poll(master_fd,
  100ms)` → `read` → `Terminal::feed` → drain `pending_responses` back to the
  fd. It works on the raw fd captured at spawn and **never holds the session
  mutex while polling** — an earlier draft held the mutex across `poll()`,
  which starved concurrent writers for up to 11 s (found by the acceptance
  test `write_reaches_shell`, fixed by the fd-level redesign).
- The **UI poll loop** (coroutine) calls `nativePoll(id, lastSeq, timeout)`:
  cheap compare under the terminal mutex, returns the new seq when output was
  parsed. The UI then fetches one binary snapshot.
- **Snapshot format**: `[u64 seq][u16 cols][u16 rows][u16 cur_row][u16
  cur_col][u8 mode_bits]` + per cell `[u32 cp][u8 n_marks]([u32 mark]…)
  [u32 fg][u32 bg][u16 attrs]`. `fg/bg = 0xFFFFFFFF` means "scheme default";
  `0x01xxxxxx` means indexed.

## Why not UniFFI

Decided against UniFFI: the surface is tiny (~20 functions + one byte-array
snapshot format) and JNI with plain types keeps the cdylib free of extra
codegen deps; the byte protocol is fully specified and unit-tested on both
sides (snapshot round-trip test in Rust, parser test in Kotlin androidTest).

## Session lifecycle

1. `nativeCreate(cols, rows, capacity)` → session id.
2. `nativeSpawn(id, cmd, args, env, cwd)` → `openpty` + `fork`: child does
   `setsid`, `TIOCSCTTY`, `dup2`, resets signal dispositions, `execve`
   (127 on failure is visible to the session). Parent sets master nonblock
   and starts the reader thread.
3. UI drives `nativePoll`/`nativeSnapshot`; input via `nativeWrite` /
   `nativePaste` (bracketed paste honored from mode 2004).
4. `nativeResize` → `TIOCSWINSZ` + `kill(-pgid, SIGWINCH)` + grid/history
   re-wrap (per-row wrap flags).
5. `nativeDestroy` → `SIGHUP` → grace 300 ms → `SIGKILL`, `waitpid`, close
   fd, bounded join of the reader, drop listener global ref.

## Reproducible build inputs

| Component | Version | Source |
|---|---|---|
| Rust toolchain | 1.83.0 | rustup pinned in CI |
| PRoot | 5.5.0 | proot-me GitHub tag |
| talloc | 2.4.2 | samba.org tarball (PRoot dependency) |
| Alpine | 3.24.4 | dl-cdn.alpinelinux.org + upstream SHA-256 |
| AGP / Kotlin | 8.7.3 / 2.0.21 | gradle/libs.versions.toml |
| Compose BOM | 2024.12.01 | gradle/libs.versions.toml |
| Gradle | 8.11.1 | wrapper properties |
