package com.xrust.app.terminal

import android.view.KeyEvent
import com.xrust.app.nativejni.TerminalNative

/**
 * Maps Android key events to PTY byte sequences via the Rust keymap.
 * Modifier codes follow xterm: 1 + shift(1) + alt(2) + ctrl(4).
 */
object KeyCodec {
    const val MOD_SHIFT = 1
    const val MOD_ALT = 2
    const val MOD_CTRL = 4

    fun modsOf(event: KeyEvent): Int {
        var m = 0
        if (event.isShiftPressed) m += MOD_SHIFT
        if (event.isAltPressed) m += MOD_ALT
        if (event.isCtrlPressed) m += MOD_CTRL
        return m
    }

    fun encode(event: KeyEvent, appCursor: Boolean, appKeypad: Boolean): ByteArray? {
        val mods = modsOf(event)
        return when (event.keyCode) {
            KeyEvent.KEYCODE_ENTER -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ENTER, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_DEL, KeyEvent.KEYCODE_BACKSPACE -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_BACKSPACE, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_TAB -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_TAB, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_ESCAPE -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ESCAPE, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_MOVE_HOME -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_HOME, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_MOVE_END -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_END, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_PAGE_UP -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_PAGE_UP, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_PAGE_DOWN -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_PAGE_DOWN, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_INSERT -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_INSERT, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_FORWARD_DEL -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_DELETE, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_DPAD_UP -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ARROW_UP, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_DPAD_DOWN -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ARROW_DOWN, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_DPAD_LEFT -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ARROW_LEFT, mods, appCursor, appKeypad)
            KeyEvent.KEYCODE_DPAD_RIGHT -> TerminalNative.nativeEncodeKey(TerminalNative.KEY_ARROW_RIGHT, mods, appCursor, appKeypad)
            in KeyEvent.KEYCODE_F1..KeyEvent.KEYCODE_F12 ->
                TerminalNative.nativeEncodeKey(
                    TerminalNative.KEY_FN_BASE + (event.keyCode - KeyEvent.KEYCODE_F1) + 1,
                    mods, appCursor, appKeypad,
                )
            else -> {
                val ch = event.unicodeChar
                if (ch > 0) TerminalNative.nativeEncodeChar(ch, mods, appCursor, appKeypad) else null
            }
        }
    }
}
