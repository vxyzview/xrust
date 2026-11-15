package com.xrust.app.shell

/**
 * Host shell sessions run `/system/bin/sh` directly — no PRoot, no rootfs.
 *
 * Known limits (documented by design, see docs/HOST_SHELL_LIMITS.md):
 *  - ToyBox sh only: no apk, no musl, no glibc software.
 *  - SELinux confines the app UID; only app-readable paths are visible.
 *  - Some /proc and /sys nodes are hidden; ioctl-heavy tools may degrade.
 *  - The terminal is still a real PTY: ncurses tools that ship with Android
 *    (e.g. `top`, `logcat`) work with TERM=xterm-256color.
 */
object HostShell {
    const val COMMAND = "/system/bin/sh"
    const val DOCUMENTATION_NOTE =
        "Host sessions are intentionally restricted by Android sandboxing. " +
            "Install the Alpine rootfs for a full userland."
}
