package com.xrust.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.xrust.app.nativejni.TerminalNative
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Smoke test on a real device/emulator: loads the Rust cdylib, creates a
 * session bound to a real PTY, and verifies the write→parse→snapshot path.
 */
@RunWith(AndroidJUnit4::class)
class NativeSmokeTest {

    @Before
    fun loadLib() {
        TerminalNative.loadLibrary()
    }

    @Test
    fun createWriteSnapshot() {
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        assertTrue(ctx != null)
        val id = TerminalNative.nativeCreate(80, 24, 1000)
        try {
            // Spawn a tiny shell that echoes and exits.
            val ok = TerminalNative.nativeSpawn(
                id, "/system/bin/sh",
                arrayOf("-c", "echo smoke-ok; exit 0"),
                arrayOf("TERM=xterm-256color", "PATH=/system/bin"),
                "/",
            )
            assertTrue("spawn must succeed", ok)
            // Poll for output.
            var seq = TerminalNative.nativeSeq(id)
            val deadline = System.currentTimeMillis() + 10_000
            var text = ""
            while (System.currentTimeMillis() < deadline) {
                seq = TerminalNative.nativePoll(id, seq, 200)
                if (seq == -1L) break
                val snap = TerminalNative.parseSnapshot(TerminalNative.nativeSnapshot(id) ?: break) ?: continue
                text = (0 until snap.rows).joinToString("\n") { r -> TerminalNative.rowText(snap, r) }
                if (text.contains("smoke-ok")) break
            }
            assertTrue("output must contain marker, got: $text", text.contains("smoke-ok"))
            // Wait for exit status.
            val deadline2 = System.currentTimeMillis() + 10_000
            while (System.currentTimeMillis() < deadline2 && TerminalNative.nativeIsAlive(id)) {
                Thread.sleep(50)
            }
            assertEquals(0, TerminalNative.nativeExitCode(id))
        } finally {
            TerminalNative.nativeDestroy(id)
        }
    }

    @Test
    fun snapshotParsingIsBounded() {
        val id = TerminalNative.nativeCreate(40, 10, 100)
        try {
            val bytes = TerminalNative.nativeSnapshot(id)!!
            // Header 17 bytes + 400 cells × 15 bytes = 6017.
            assertEquals(17 + 400 * 15, bytes.size)
            val snap = TerminalNative.parseSnapshot(bytes)!!
            assertEquals(40, snap.cols)
            assertEquals(10, snap.rows)
            assertTrue(snap.cursorVisible)
        } finally {
            TerminalNative.nativeDestroy(id)
        }
    }
}
