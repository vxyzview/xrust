package com.xrust.app

import com.xrust.app.data.Profile
import com.xrust.app.data.ProfilesStore
import com.xrust.app.installer.HashVerify
import com.xrust.app.ui.theme.ALL_TERMINAL_SCHEMES
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import java.nio.file.Path

class ProfilesStoreTest {
    @TempDir
    lateinit var tempDir: Path

    private val store = ProfilesStore { tempDir.resolve("profiles.json").toFile() }

    @Test
    fun `defaults when file missing`() {
        assertTrue(store.load().size >= 2)
    }

    @Test
    fun `save and load roundtrip`() {
        val p = Profile(id = "x", name = "Test", env = mapOf("FOO" to "bar"), args = listOf("-l"))
        store.save(listOf(p))
        val loaded = store.load()
        assertEquals(1, loaded.size)
        assertEquals("bar", loaded[0].env["FOO"])
    }

    @Test
    fun `corrupt file falls back to defaults`() {
        tempDir.resolve("profiles.json").toFile().writeText("{ not json")
        assertTrue(store.load().size >= 2, "corrupt store must not crash")
    }

    @Test
    fun `export import roundtrip`() {
        val profiles = store.defaultProfiles()
        val json = store.export(profiles)
        val back = store.import(json)
        assertEquals(profiles, back)
    }

    @Test
    fun `import rejects garbage`() {
        org.junit.jupiter.api.Assertions.assertThrows(Exception::class.java) {
            store.import("not-json-at-all")
        }
    }
}

class HashVerifyTest {
    @Test
    fun `sha256 of known input`() {
        // sha256("abc")
        assertEquals(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            HashVerify.sha256Hex("abc".toByteArray()),
        )
    }

    @Test
    fun `matches is case and prefix tolerant`() {
        assertFalse(HashVerify.matches("BA7816BF-nope", "abc".toByteArray()))
        assertTrue(
            HashVerify.matches(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                "abc".toByteArray(),
            ),
        )
        assertTrue(
            HashVerify.matches(
                "sha256:BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
                "abc".toByteArray(),
            ),
        )
    }
}

class SchemeValidationTest {
    @Test
    fun `all built-in schemes have valid contrast and unique names`() {
        val names = ALL_TERMINAL_SCHEMES.map { it.name }
        assertEquals(names.size, names.toSet().size, "scheme names must be unique")
        assertTrue(names.size >= 12)
        for (s in ALL_TERMINAL_SCHEMES) {
            assertFalse(s.foreground == s.background, s.name)
            assertTrue(s.palette.size == 16, s.name)
            // Luminance ordering sanity for dark schemes.
            if (s.isDark) {
                assertTrue(luma(s.background) < luma(s.foreground), s.name)
            } else {
                assertTrue(luma(s.background) > luma(s.foreground), s.name)
            }
        }
    }

    @Test
    fun `amoled scheme is pure black`() {
        val amoled = ALL_TERMINAL_SCHEMES.first { it.name == "AMOLED Pure Black" }
        assertEquals(0x000000, amoled.background)
    }

    private fun luma(c: Int): Double =
        0.2126 * ((c shr 16) and 0xFF) + 0.7152 * ((c shr 8) and 0xFF) + 0.0722 * (c and 0xFF)
}

class FileManagerPathsTest {
    @TempDir
    lateinit var tempDir: Path

    @Test
    fun `list sorts dirs first`() {
        val root = tempDir.toFile()
        File(root, "b.txt").writeText("x")
        File(root, "a").mkdirs()
        val fm = FileManager { root }
        val entries = fm.list("side", null)
        assertTrue(entries.first().isDir)
    }

    @Test
    fun `mkdir sanitizes slashes`() {
        val root = tempDir.toFile()
        val fm = FileManager { root }
        assertTrue(fm.mkdir("side", root.absolutePath, "a/b/../c"))
        // No traversal: only a single sanitized directory created.
        assertEquals(1, root.listFiles()!!.size)
    }
}
