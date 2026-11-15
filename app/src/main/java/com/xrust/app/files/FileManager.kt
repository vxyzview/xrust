package com.xrust.app.files

import java.io.File

/**
 * Dual-pane file model: one side is the Android storage visible to the app,
 * the other is the Alpine rootfs (guest). All operations are plain java.io /
 * java.nio — no shell commands, no root required.
 */
data class FileEntry(
    val path: String,
    val name: String,
    val isDir: Boolean,
    val size: Long,
    val lastModified: Long,
)

class FileManager(private val roots: (String) -> File) {

    fun list(side: String, path: String?): List<FileEntry> {
        val base = roots(side)
        val dir = if (path.isNullOrBlank()) base else File(path)
        val safe = if (dir.canonicalPath.startsWith(base.canonicalPath)) dir else base
        return safe.listFiles()
            ?.map { FileEntry(it.absolutePath, it.name, it.isDirectory, it.length(), it.lastModified()) }
            ?.sortedWith(compareByDescending<FileEntry> { it.isDir }.thenBy { it.name.lowercase() })
            ?: emptyList()
    }

    fun copy(side: String, src: String, destDir: String): Boolean {
        val f = File(src)
        val dst = File(destDir, f.name)
        if (!dst.canonicalPath.startsWith(roots(side).canonicalPath)) return false
        return if (f.isDirectory) f.copyRecursively(dst, overwrite = true) else f.copyTo(dst, overwrite = true).exists()
    }

    fun move(side: String, src: String, destDir: String): Boolean {
        val f = File(src)
        val dst = File(destDir, f.name)
        if (!dst.canonicalPath.startsWith(roots(side).canonicalPath)) return false
        return f.renameTo(dst)
    }

    fun delete(path: String): Boolean = File(path).let { f ->
        if (f.isDirectory) f.deleteRecursively() else f.delete()
    }

    fun mkdir(side: String, parent: String, name: String): Boolean {
        val sanitized = name.replace(Regex("[/\\\\]"), "_")
        return File(parent, sanitized).mkdirs()
    }

    fun readText(path: String, maxBytes: Long = 2_000_000): String? {
        val f = File(path)
        if (!f.isFile || f.length() > maxBytes) return null
        return runCatching { f.readText() }.getOrNull()
    }

    fun writeText(path: String, text: String): Boolean =
        runCatching { File(path).writeText(text); true }.getOrDefault(false)
}
