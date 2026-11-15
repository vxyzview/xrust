package com.xrust.app.terminal

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.xrust.app.nativejni.TerminalNative
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Owns one Rust terminal session: lifecycle, the poll/render loop, and input.
 */
class TerminalController(
    val id: Long,
    initialCols: Int,
    initialRows: Int,
    scrollback: Int,
) {
    var snapshot by mutableStateOf(TerminalNative.parseSnapshot(TerminalNative.nativeSnapshot(id)))
        private set
    var title by mutableStateOf("")
        private set
    var cwd by mutableStateOf<String?>(null)
        private set
    var exitCode by mutableStateOf<Int?>(null)
        private set
    var bellTick by mutableStateOf(0)
        private set

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var pollJob: Job? = null

    fun start() {
        if (pollJob != null) return
        pollJob = scope.launch { pollLoop() }
    }

    private suspend fun pollLoop() {
        var lastSeq = TerminalNative.nativeSeq(id)
        while (true) {
            val seq = withContext(Dispatchers.Default) { TerminalNative.nativePoll(id, lastSeq, 200) }
            if (seq == -1L) {
                exitCode = TerminalNative.nativeExitCode(id).takeIf { it != Int.MIN_VALUE } ?: 0
                refresh()
                break
            }
            if (seq != lastSeq) {
                lastSeq = seq
                refresh()
            }
        }
    }

    private suspend fun refresh() = withContext(Dispatchers.Default) {
        snapshot = TerminalNative.parseSnapshot(TerminalNative.nativeSnapshot(id))
        title = TerminalNative.nativeGetTitle(id) ?: title
    }

    fun resize(cols: Int, rows: Int) = TerminalNative.nativeResize(id, cols, rows)

    fun send(bytes: ByteArray) = TerminalNative.nativeWrite(id, bytes)

    fun send(text: String) = TerminalNative.nativeWriteString(id, text)

    fun paste(text: String) = TerminalNative.nativePaste(id, text)

    fun signal(sig: Int) = TerminalNative.nativeSignal(id, sig)

    fun setScrollback(cap: Int) = TerminalNative.nativeSetScrollback(id, cap)

    fun clearScrollback() = TerminalNative.nativeClearScrollback(id)

    fun setListener(listener: TerminalNative.Listener?) = TerminalNative.nativeSetListener(id, listener)

    fun destroy() {
        scope.cancel()
        TerminalNative.nativeDestroy(id)
    }
}
