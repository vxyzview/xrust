package com.xrust.app.util

import android.app.Application
import android.os.Process
import java.io.File

/**
 * Last-resort crash handler: writes a small tombstone to filesDir and exits.
 * Keeps the terminal from dying silently during long shell sessions.
 */
object CrashGuard {
    fun install(app: Application) {
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            runCatching {
                val dir = File(app.filesDir, "crash").apply { mkdirs() }
                File(dir, "crash-${System.currentTimeMillis()}.txt").writeText(
                    buildString {
                        append("thread=").append(thread.name).append('\n')
                        append(throwable.stackTraceToString())
                    },
                )
                // Trim to the newest 5 reports.
                dir.listFiles()?.sortedByDescending { it.name }?.drop(5)?.forEach { it.delete() }
            }
            previous?.uncaughtException(thread, throwable) ?: Process.killProcess(Process.myPid())
        }
    }
}
