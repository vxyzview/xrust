package com.xrust.app.installer

import java.security.MessageDigest

object HashVerify {
    fun sha256Hex(bytes: ByteArray): String =
        MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }

    /**
     * Upstream checksum files are "<hash>␣␣<filename>" on one line — take the
     * first whitespace-separated token only, or the filename rides along and
     * every comparison fails.
     */
    fun matches(expected: String, actual: ByteArray): Boolean =
        expected.trim().split(Regex("\\s+")).first().removePrefix("sha256:")
            .equals(sha256Hex(actual), ignoreCase = true)
}
