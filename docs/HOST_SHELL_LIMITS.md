# Host shell limits

Host sessions (profile `kind: "host"`) run Android's own `/system/bin/sh`
(ToyBox) directly, inside the app's Linux UID. There is intentionally **no
root, no chroot, no privilege escalation** — everything below follows from
that.

## What works

- A real PTY: line editing, `TERM=xterm-256color`, colors, cursor keys.
- ToyBox applets present on the device: `ls`, `cat`, `top`, `logcat`
  (app-visible buffers only), `ps`, `df`, `ping`, `wget` (subject to
  network policy), `vi` (ToyBox build), etc.
- Scripts placed under the app's own storage.

## What does NOT work (by design)

| Missing | Reason |
|---|---|
| `apk add …` | No Alpine userland on host sessions; install the rootfs. |
| glibc software | Android has bionic, not glibc. |
| chroot / mount | Requires root; never attempted. |
| Reading other apps' data | SELinux + UID isolation. |
| Full `/proc`, `/sys` | Kernel + SELinux hide most nodes from apps. |

## Not a security boundary statement

The host shell is a convenience for debugging the device, not a sandbox
feature. The Alpine rootfs session (PRoot) is likewise *not* a security
boundary — PRoot is ptrace-based userspace virtualization, suitable for
running userland software, not for containing malicious code.
