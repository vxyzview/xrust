package com.xrust.app.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

/** A launch profile: how a session starts. */
@Serializable
data class Profile(
    val id: String,
    val name: String,
    /** "alpine" (via proot) or "host" (restricted /system/bin/sh). */
    val kind: String = "alpine",
    val command: String = "/bin/sh",
    val args: List<String> = listOf("-l"),
    val env: Map<String, String> = emptyMap(),
    val cwd: String = "/root",
    val colorScheme: String? = null,
    val fontSizeSp: Int? = null,
    val extraKeysLayout: String? = null,
)

class ProfilesStore(private val fileProvider: () -> java.io.File) {

    private val json = Json { ignoreUnknownKeys = true; prettyPrint = true }

    fun load(): List<Profile> {
        val f = fileProvider()
        if (!f.exists()) return defaultProfiles()
        return runCatching {
            json.decodeFromString<List<Profile>>(f.readText())
        }.getOrElse { defaultProfiles() }
    }

    fun save(profiles: List<Profile>) {
        fileProvider().writeText(json.encodeToString(profiles))
    }

    fun defaultProfiles(): List<Profile> = listOf(
        Profile(
            id = "alpine-default",
            name = "Alpine sh",
            kind = "alpine",
            command = "/bin/sh",
            args = listOf("-l"),
            cwd = "/root",
        ),
        Profile(
            id = "host-default",
            name = "Host shell (restricted)",
            kind = "host",
            command = "/system/bin/sh",
            args = emptyList(),
            cwd = "/",
        ),
    )

    /** Export profiles as JSON (for the Files screen / share). */
    fun export(profiles: List<Profile>): String = json.encodeToString(profiles)

    /** Import profiles from JSON; unknown keys ignored, invalid input rejected. */
    fun import(text: String): List<Profile> = json.decodeFromString(text)
}
