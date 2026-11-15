package com.xrust.app.proot

import android.content.Context
import com.xrust.app.data.Profile
import java.io.File

/**
 * Assembles the PRoot command line that turns the Alpine rootfs into a
 * working root filesystem. The app ships a PRoot binary built from the
 * proot-me v5.5.0 sources for each ABI (packaged as libproot.so), extracted
 * to filesDir/bin on first run.
 */
object ProotLauncher {

    fun prootBinary(context: Context): File {
        val dir = File(context.filesDir, "bin").apply { mkdirs() }
        val f = File(dir, "proot")
        if (!f.exists()) {
            // jniLibs trick: ship the binary as libproot.so so the APK
            // packaging keeps exec bits; copy it out on first run.
            val abi = android.os.Build.SUPPORTED_ABIS.firstOrNull() ?: "arm64-v8a"
            val src = File(context.applicationInfo.nativeLibraryDir, "libproot.so")
            if (src.exists()) {
                src.copyTo(f, overwrite = true)
            } else {
                throw IllegalStateException("proot binary missing for abi $abi")
            }
        }
        if (!f.canExecute()) f.setExecutable(true, false)
        return f
    }

    /** Build argv for a profile. Host profiles bypass proot entirely. */
    fun argv(context: Context, profile: Profile): List<String> {
        if (profile.kind == "host") {
            return listOf(profile.command) + profile.args
        }
        val rootfs = File(context.filesDir, "alpine")
        require(rootfs.resolve("etc/alpine-release").exists()) {
            "Alpine rootfs is not installed yet"
        }
        val proot = prootBinary(context)
        val args = mutableListOf(
            proot.absolutePath,
            "--kill-on-exit",
            "-R", rootfs.absolutePath,
            // Host accesses the terminal needs.
            "-b", "/proc",
            "-b", "/dev",
            "-b", "/sys",
            // App storage visible inside the guest for file transfers.
            "-b", "${context.filesDir}/share:/root/share",
        )
        args += profile.command
        args += profile.args
        return args
    }

    /** Env for the spawned process. */
    fun env(profile: Profile): List<String> {
        val base = mutableListOf(
            "HOME=/root",
            "USER=root",
            "TERM=xterm-256color",
            "COLORTERM=truecolor",
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "LANG=C.UTF-8",
        )
        profile.env.forEach { (k, v) -> base += "$k=$v" }
        return base
    }
}
