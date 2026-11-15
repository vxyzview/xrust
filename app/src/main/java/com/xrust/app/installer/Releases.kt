package com.xrust.app.installer

/** Known Alpine releases we can install (x86_64 / arm64 / armv7). */
object Releases {
    data class Release(val version: String, val arch: String, val url: String, val sha256Url: String)

    private const val CDN = "https://dl-cdn.alpinelinux.org/alpine"

    fun forAbi(abi: String): Release {
        val arch = when (abi) {
            "arm64-v8a" -> "aarch64"
            "armeabi-v7a" -> "armv7"
            "x86_64" -> "x86_64"
            "x86" -> "x86"
            else -> "aarch64"
        }
        val base = "$CDN/v3.24/releases/$arch"
        return Release(
            version = "3.24.4",
            arch = arch,
            url = "$base/alpine-minirootfs-3.24.4-$arch.tar.gz",
            sha256Url = "$base/alpine-minirootfs-3.24.4-$arch.tar.gz.sha256",
        )
    }
}
