package com.xrust.app.installer

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import java.io.File
import java.io.IOException
import java.util.concurrent.TimeUnit

/**
 * Downloads and verifies the Alpine minirootfs, then extracts it safely under
 * filesDir/alpine (no shell, no symlinks escaping the target dir, path
 * traversal rejected).
 */
class AlpineInstaller(private val context: Context) {

    sealed interface Progress {
        data class Downloading(val percent: Int) : Progress
        data class Verifying(val size: Long) : Progress
        data class Extracting(val entries: Int) : Progress
        data class Done(val dir: File) : Progress
        data class Failed(val message: String) : Progress
    }

    private val client = OkHttpClient.Builder()
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .build()

    val rootfsDir: File get() = File(context.filesDir, "alpine")
    val isInstalled: Boolean get() = File(rootfsDir, "etc/alpine-release").exists()

    fun installedVersion(): String? =
        File(rootfsDir, "etc/alpine-release").takeIf { it.exists() }?.readText()?.trim()

    /** Download + verify + extract. `onProgress` runs on the caller thread. */
    suspend fun install(abi: String, onProgress: (Progress) -> Unit): File = withContext(Dispatchers.IO) {
        val release = Releases.forAbi(abi)
        try {
            rootfsDir.deleteRecursively()
            val tmpTar = File(context.cacheDir, "alpine.tar.gz")

            // -- download with progress --
            val req = Request.Builder().url(release.url).build()
            client.newCall(req).execute().use { resp ->
                if (!resp.isSuccessful) throw IOException("download failed: HTTP ${resp.code}")
                val body = resp.body ?: throw IOException("empty body")
                val total = body.contentLength()
                var read = 0L
                tmpTar.outputStream().use { out ->
                    val src = body.byteStream()
                    val buf = ByteArray(64 * 1024)
                    while (true) {
                        val n = src.read(buf)
                        if (n < 0) break
                        out.write(buf, 0, n)
                        read += n
                        if (total > 0) onProgress(Progress.Downloading((read * 100 / total).toInt()))
                    }
                }
            }

            // -- verify SHA-256 against dl-cdn --
            onProgress(Progress.Verifying(tmpTar.length()))
            val shaReq = Request.Builder().url(release.sha256Url).build()
            val expected = client.newCall(shaReq).execute().use { r ->
                if (!r.isSuccessful) throw IOException("checksum fetch failed: HTTP ${r.code}")
                r.body?.string()?.trim() ?: throw IOException("empty checksum")
            }
            val bytes = tmpTar.readBytes()
            if (!HashVerify.matches(expected, bytes)) {
                throw IOException("SHA-256 mismatch: got ${HashVerify.sha256Hex(bytes)}, want $expected")
            }

            // -- safe extract (no shell; reject traversal) --
            onProgress(Progress.Extracting(0))
            val dest = rootfsDir
            dest.mkdirs()
            var entries = 0
            java.util.zip.GZIPInputStream(bytes.inputStream()).use { gz ->
                TarArchiveInputStream(gz).use { tar ->
                    while (true) {
                        val entry = tar.nextTarEntry ?: break
                        val name = entry.name
                        if (name.contains("..") || name.startsWith("/") || name.contains('\0')) {
                            throw IOException("unsafe tar entry: $name")
                        }
                        val target = File(dest, name)
                        if (!target.canonicalPath.startsWith(dest.canonicalPath + File.separator)) {
                            throw IOException("traversal blocked: $name")
                        }
                        if (entry.isDirectory) {
                            target.mkdirs()
                        } else {
                            target.parentFile?.mkdirs()
                            // Reject absolute symlinks; rewrite relative ones into the rootfs.
                            if (entry.isSymbolicLink) {
                                val link = entry.linkName
                                if (link.startsWith("/")) {
                                    throw IOException("absolute symlink rejected: $name -> $link")
                                }
                                runCatching { target.delete() }
                                java.nio.file.Files.createSymbolicLink(
                                    target.toPath(),
                                    java.nio.file.Path.of(link),
                                )
                            } else {
                                target.outputStream().use { tar.copyTo(it) }
                            }
                        }
                        entries++
                        if (entries % 200 == 0) onProgress(Progress.Extracting(entries))
                    }
                }
            }
            File(dest, "etc/alpine-release").takeIf { it.exists() }
                ?: throw IOException("extraction finished but etc/alpine-release is missing")
            tmpTar.delete()
            onProgress(Progress.Done(dest))
            dest
        } catch (e: Exception) {
            onProgress(Progress.Failed(e.message ?: e.javaClass.simpleName))
            throw e
        }
    }

    /** Write /etc/profile.d enhancements: prompt with OSC 7 cwd reporting. */
    fun writeProfileInit() {
        val dir = File(rootfsDir, "etc/profile.d")
        dir.mkdirs()
        val script = buildString {
            appendLine("# xrust integration.")
            appendLine("# Report cwd via OSC 7 so the app's file manager can sync.")
            appendLine("""__at_cwd() { printf '\033]7;file://local%s\007' "$(pwd 2>/dev/null)"; }""")
            appendLine("# ash/dash has no PROMPT_COMMAND; emit the OSC from PS1 instead.")
            appendLine("""PS1='$(__at_cwd)\[\033[01;32m\]xrust\[\033[00m\]:\w\$ '""")
        }
        File(dir, "xrust.sh").writeText(script)
    }
}
