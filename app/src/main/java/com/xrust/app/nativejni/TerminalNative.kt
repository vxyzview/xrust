package com.xrust.app.nativejni

import java.nio.ByteBuffer

/**
 * JNI boundary to the Rust `xrust_core` cdylib. All terminal logic lives
 * in Rust; this class only marshals calls and parses snapshots.
 */
object TerminalNative {

    /** Listener fired from the Rust reader thread. */
    interface Listener {
        fun onTitle(title: String)
        fun onBell()
        fun onExit(code: Int)
        fun onCwd(cwd: String)
    }

    // Key code constants mirrored from android_jni.rs.
    const val KEY_ENTER = 1
    const val KEY_BACKSPACE = 2
    const val KEY_TAB = 3
    const val KEY_ESCAPE = 4
    const val KEY_HOME = 5
    const val KEY_END = 6
    const val KEY_PAGE_UP = 7
    const val KEY_PAGE_DOWN = 8
    const val KEY_INSERT = 9
    const val KEY_DELETE = 10
    const val KEY_ARROW_UP = 11
    const val KEY_ARROW_DOWN = 12
    const val KEY_ARROW_LEFT = 13
    const val KEY_ARROW_RIGHT = 14
    const val KEY_FN_BASE = 20

    @JvmStatic
    external fun nativeCreate(cols: Int, rows: Int, capacity: Int): Long

    @JvmStatic
    external fun nativeSpawn(id: Long, cmd: String, args: Array<String>, env: Array<String>, cwd: String): Boolean

    @JvmStatic
    external fun nativeWrite(id: Long, bytes: ByteArray)

    @JvmStatic
    external fun nativeWriteString(id: Long, text: String)

    @JvmStatic
    external fun nativePaste(id: Long, text: String)

    @JvmStatic
    external fun nativeEncodeChar(ch: Int, mods: Int, appCursor: Boolean, appKeypad: Boolean): ByteArray?

    @JvmStatic
    external fun nativeEncodeKey(key: Int, mods: Int, appCursor: Boolean, appKeypad: Boolean): ByteArray?

    @JvmStatic
    external fun nativeResize(id: Long, cols: Int, rows: Int)

    @JvmStatic
    external fun nativeSnapshot(id: Long): ByteArray?

    @JvmStatic
    external fun nativePoll(id: Long, lastSeq: Long, timeoutMs: Int): Long

    @JvmStatic
    external fun nativeSeq(id: Long): Long

    @JvmStatic
    external fun nativeIsAlive(id: Long): Boolean

    @JvmStatic
    external fun nativeExitCode(id: Long): Int

    @JvmStatic
    external fun nativeSignal(id: Long, sig: Int)

    @JvmStatic
    external fun nativeSetListener(id: Long, listener: Listener?)

    @JvmStatic
    external fun nativeSetScrollback(id: Long, cap: Int)

    @JvmStatic
    external fun nativeClearScrollback(id: Long)

    @JvmStatic
    external fun nativeGetTitle(id: Long): String?

    @JvmStatic
    external fun nativeGetHistoryLen(id: Long): Int

    @JvmStatic
    external fun nativeDestroy(id: Long)

    @JvmStatic
    external fun nativeDestroyAll()

    /** One rendered frame: cell codepoints + per-cell style. */
    data class Snapshot(
        val seq: Long,
        val cols: Int,
        val rows: Int,
        val cursorRow: Int,
        val cursorCol: Int,
        val cursorVisible: Boolean,
        val altScreen: Boolean,
        val cells: IntArray,      // codepoints
        val fg: IntArray,         // 0xFFFFFFFF = default
        val bg: IntArray,
        val attrs: IntArray,      // bit flags matching Rust ATTR_*
    ) {
        override fun equals(other: Any?): Boolean = this === other
        override fun hashCode(): Int = seq.hashCode()
    }

    /**
     * Parse the Rust snapshot format:
     * `[u64 seq][u16 cols][u16 rows][u16 cur_row][u16 cur_col][u8 mode_bits]`
     * then per cell: `[u32 cp][u8 n_marks]([u32 mark]…)[u32 fg][u32 bg][u16 attrs]`.
     */
    fun parseSnapshot(bytes: ByteArray): Snapshot? {
        if (bytes.size < 17) return null
        val buf = ByteBuffer.wrap(bytes)
        val seq = buf.long
        val cols = buf.short.toInt() and 0xFFFF
        val rows = buf.short.toInt() and 0xFFFF
        val curRow = buf.short.toInt() and 0xFFFF
        val curCol = buf.short.toInt() and 0xFFFF
        val bits = buf.get().toInt() and 0xFF
        val n = cols * rows
        val cells = IntArray(n)
        val fgs = IntArray(n)
        val bgs = IntArray(n)
        val attrs = IntArray(n)
        for (i in 0 until n) {
            val cp = buf.int
            cells[i] = cp
            val marks = buf.get().toInt() and 0xFF
            repeat(marks) { buf.int } // combining marks are rendered with the lead cell
            fgs[i] = buf.int
            bgs[i] = buf.int
            attrs[i] = buf.short.toInt() and 0xFFFF
        }
        return Snapshot(
            seq = seq,
            cols = cols,
            rows = rows,
            cursorRow = curRow,
            cursorCol = curCol,
            cursorVisible = bits and 1 != 0,
            altScreen = bits and 2 != 0,
            cells = cells,
            fg = fgs,
            bg = bgs,
            attrs = attrs,
        )
    }

    /** Build the cell text for one row from a snapshot (wide-cont skipped). */
    fun rowText(s: Snapshot, row: Int): String {
        val sb = StringBuilder(s.cols)
        val base = row * s.cols
        for (c in 0 until s.cols) {
            val i = base + c
            if (s.attrs[i] and 0x100 != 0) continue // ATTR_WIDE_CONT
            val cp = s.cells[i]
            if (cp == 0) continue
            sb.appendCodePoint(cp)
        }
        return sb.toString()
    }

    fun loadLibrary() {
        System.loadLibrary("xrust_core")
    }
}
