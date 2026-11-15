package com.xrust.app.ui.theme

import androidx.compose.ui.graphics.Color

/** App chrome colors for the four theme modes. */
data class ThemeColors(
    val name: String,
    val isDark: Boolean,
    val primary: Color,
    val background: Color,
    val surface: Color,
    val surfaceVariant: Color,
)

val ALPINE_DARK = ThemeColors(
    "Alpine Dark", isDark = true,
    primary = Color(0xFF4FC3F7),
    background = Color(0xFF0E1218),
    surface = Color(0xFF151B24),
    surfaceVariant = Color(0xFF1C2430),
)

val ALPINE_LIGHT = ThemeColors(
    "Alpine Light", isDark = false,
    primary = Color(0xFF0B57D0),
    background = Color(0xFFF4F6F9),
    surface = Color(0xFFFFFFFF),
    surfaceVariant = Color(0xFFE7EBF1),
)

/** AMOLED: true #000000, no elevation tint. */
val AMOLED_BLACK = ThemeColors(
    "AMOLED Pure Black", isDark = true,
    primary = Color(0xFF82AAFF),
    background = Color(0xFF000000),
    surface = Color(0xFF000000),
    surfaceVariant = Color(0xFF0D0D0D),
)

val ROSE_ASH = ThemeColors(
    "Rose Ash", isDark = true,
    primary = Color(0xFFEB8CAC),
    background = Color(0xFF191417),
    surface = Color(0xFF201A1E),
    surfaceVariant = Color(0xFF2A2226),
)

val ALL_APP_THEMES = listOf(ALPINE_DARK, ALPINE_LIGHT, AMOLED_BLACK, ROSE_ASH)
