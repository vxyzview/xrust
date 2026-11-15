package com.xrust.app.installer

import java.security.MessageDigest

object HashVerify {
    fun sha256Hex(bytes: ByteArray): String =
        MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }

    /** Case-insensitive comparison against the upstream checksum text. */
    fun matches(expected: String, actual: ByteArray): Boolean =
        expected.removePrefix("sha256:").trim().equals(sha256Hex(actual), ignoreCase = true)
}
