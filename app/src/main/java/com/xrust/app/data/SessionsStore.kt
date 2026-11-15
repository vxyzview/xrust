package com.xrust.app.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

/** Persisted metadata about open sessions so they can be restored in-app. */
@Serializable
data class SessionMeta(
    val id: Long,
    val profileId: String,
    val title: String,
    val createdAt: Long,
)

object SessionsStore {
    private val json = Json { ignoreUnknownKeys = true }

    fun load(file: java.io.File): List<SessionMeta> =
        if (!file.exists()) emptyList()
        else runCatching { json.decodeFromString<List<SessionMeta>>(file.readText()) }.getOrElse { emptyList() }

    fun save(file: java.io.File, sessions: List<SessionMeta>) {
        file.writeText(json.encodeToString(sessions))
    }
}
