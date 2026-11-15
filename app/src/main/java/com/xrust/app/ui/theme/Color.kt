package com.xrust.app.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

/**
 * App chrome themes for the four theme modes. Each theme carries a COMPLETE
 * Material 3 ColorScheme — every tonal role is set explicitly, so screens
 * never fall back to the framework's light-scheme defaults (the source of
 * the off-palette pink/blue one-off widgets).
 */
data class ThemeColors(
    val name: String,
    val isDark: Boolean,
    val scheme: ColorScheme,
)

private val SCRIM = Color(0xFF000000)

val ALPINE_DARK = ThemeColors(
    "Alpine Dark", isDark = true,
    scheme = darkColorScheme(
        primary = Color(0xFF4FC3F7), onPrimary = Color(0xFF00344F),
        primaryContainer = Color(0xFF004B6D), onPrimaryContainer = Color(0xFFC9E8FF),
        secondary = Color(0xFFB3CADB), onSecondary = Color(0xFF1C3542),
        secondaryContainer = Color(0xFF334B59), onSecondaryContainer = Color(0xFFD3E9F8),
        tertiary = Color(0xFF9CD0B4), onTertiary = Color(0xFF003825),
        tertiaryContainer = Color(0xFF16503A), onTertiaryContainer = Color(0xFFB6EBCF),
        error = Color(0xFFFFB4AB), onError = Color(0xFF690005),
        errorContainer = Color(0xFF93000A), onErrorContainer = Color(0xFFFFDAD6),
        background = Color(0xFF0E1218), onBackground = Color(0xFFE1E5EC),
        surface = Color(0xFF0E1218), onSurface = Color(0xFFE1E5EC),
        surfaceVariant = Color(0xFF1C2430), onSurfaceVariant = Color(0xFFB9C4CE),
        outline = Color(0xFF89939E), outlineVariant = Color(0xFF3C4B58),
        inverseSurface = Color(0xFFE1E5EC), inverseOnSurface = Color(0xFF2B323A),
        inversePrimary = Color(0xFF006392),
        surfaceTint = Color(0xFF4FC3F7),
        surfaceDim = Color(0xFF0E1218), surfaceBright = Color(0xFF3A434F),
        surfaceContainerLowest = Color(0xFF090C11), surfaceContainerLow = Color(0xFF151B24),
        surfaceContainer = Color(0xFF1C2430), surfaceContainerHigh = Color(0xFF262F3B),
        surfaceContainerHighest = Color(0xFF313B47),
        scrim = SCRIM,
    ),
)

val ALPINE_LIGHT = ThemeColors(
    "Alpine Light", isDark = false,
    scheme = lightColorScheme(
        primary = Color(0xFF0B57D0), onPrimary = Color(0xFFFFFFFF),
        primaryContainer = Color(0xFFD5E3FF), onPrimaryContainer = Color(0xFF001D36),
        secondary = Color(0xFF51606E), onSecondary = Color(0xFFFFFFFF),
        secondaryContainer = Color(0xFFD4E4F2), onSecondaryContainer = Color(0xFF0F1D29),
        tertiary = Color(0xFF456A57), onTertiary = Color(0xFFFFFFFF),
        tertiaryContainer = Color(0xFFCCEAD8), onTertiaryContainer = Color(0xFF002114),
        error = Color(0xFFB3261E), onError = Color(0xFFFFFFFF),
        errorContainer = Color(0xFFF9DEDC), onErrorContainer = Color(0xFF410E0B),
        background = Color(0xFFF4F6F9), onBackground = Color(0xFF191C20),
        surface = Color(0xFFF4F6F9), onSurface = Color(0xFF191C20),
        surfaceVariant = Color(0xFFE7EBF1), onSurfaceVariant = Color(0xFF41484E),
        outline = Color(0xFF71787E), outlineVariant = Color(0xFFC1C7CE),
        inverseSurface = Color(0xFF2E3135), inverseOnSurface = Color(0xFFF0F0F4),
        inversePrimary = Color(0xFF9ECDFF),
        surfaceTint = Color(0xFF0B57D0),
        surfaceDim = Color(0xFFDADDE1), surfaceBright = Color(0xFFF4F6F9),
        surfaceContainerLowest = Color(0xFFFFFFFF), surfaceContainerLow = Color(0xFFFFFFFF),
        surfaceContainer = Color(0xFFEDEFF3), surfaceContainerHigh = Color(0xFFE7EAEE),
        surfaceContainerHighest = Color(0xFFE1E4E8),
        scrim = SCRIM,
    ),
)

/** AMOLED: true #000000 at every surface level, no elevation tint. */
val AMOLED_BLACK = ThemeColors(
    "AMOLED Pure Black", isDark = true,
    scheme = darkColorScheme(
        primary = Color(0xFF82AAFF), onPrimary = Color(0xFF002F7C),
        primaryContainer = Color(0xFF173E7C), onPrimaryContainer = Color(0xFFD8E2FF),
        secondary = Color(0xFF9EAFC7), onSecondary = Color(0xFF0A1A2E),
        secondaryContainer = Color(0xFF1A2B40), onSecondaryContainer = Color(0xFFD8E4F5),
        tertiary = Color(0xFFA5CFB8), onTertiary = Color(0xFF003825),
        tertiaryContainer = Color(0xFF144E38), onTertiaryContainer = Color(0xFFB9EFD0),
        error = Color(0xFFFFB4AB), onError = Color(0xFF690005),
        errorContainer = Color(0xFF93000A), onErrorContainer = Color(0xFFFFDAD6),
        background = Color(0xFF000000), onBackground = Color(0xFFE3E5EA),
        surface = Color(0xFF000000), onSurface = Color(0xFFE3E5EA),
        surfaceVariant = Color(0xFF101114), onSurfaceVariant = Color(0xFFB8BEC9),
        outline = Color(0xFF8A919C), outlineVariant = Color(0xFF1E2126),
        inverseSurface = Color(0xFFE3E5EA), inverseOnSurface = Color(0xFF000000),
        inversePrimary = Color(0xFF4C7FE8),
        surfaceTint = Color(0xFF82AAFF),
        surfaceDim = Color(0xFF000000), surfaceBright = Color(0xFF1A1C20),
        surfaceContainerLowest = Color(0xFF000000), surfaceContainerLow = Color(0xFF000000),
        surfaceContainer = Color(0xFF050505), surfaceContainerHigh = Color(0xFF0D0D0D),
        surfaceContainerHighest = Color(0xFF121212),
        scrim = SCRIM,
    ),
)

val ROSE_ASH = ThemeColors(
    "Rose Ash", isDark = true,
    scheme = darkColorScheme(
        primary = Color(0xFFEB8CAC), onPrimary = Color(0xFF541131),
        primaryContainer = Color(0xFF6E2946), onPrimaryContainer = Color(0xFFFFD8E1),
        secondary = Color(0xFFD5C2CA), onSecondary = Color(0xFF3B2A31),
        secondaryContainer = Color(0xFF544047), onSecondaryContainer = Color(0xFFF2DAE1),
        tertiary = Color(0xFFEBC46E), onTertiary = Color(0xFF3E2C00),
        tertiaryContainer = Color(0xFF5B4200), onTertiaryContainer = Color(0xFFFFDF9E),
        error = Color(0xFFFFB4AB), onError = Color(0xFF690005),
        errorContainer = Color(0xFF93000A), onErrorContainer = Color(0xFFFFDAD6),
        background = Color(0xFF191417), onBackground = Color(0xFFEDE0E4),
        surface = Color(0xFF191417), onSurface = Color(0xFFEDE0E4),
        surfaceVariant = Color(0xFF2A2226), onSurfaceVariant = Color(0xFFD3C4C9),
        outline = Color(0xFF9C8D93), outlineVariant = Color(0xFF4F4349),
        inverseSurface = Color(0xFFEDE0E4), inverseOnSurface = Color(0xFF352F32),
        inversePrimary = Color(0xFF8E4964),
        surfaceTint = Color(0xFFEB8CAC),
        surfaceDim = Color(0xFF191417), surfaceBright = Color(0xFF41373B),
        surfaceContainerLowest = Color(0xFF130F11), surfaceContainerLow = Color(0xFF201A1E),
        surfaceContainer = Color(0xFF2A2226), surfaceContainerHigh = Color(0xFF352C30),
        surfaceContainerHighest = Color(0xFF41373B),
        scrim = SCRIM,
    ),
)

val ALL_APP_THEMES = listOf(ALPINE_DARK, ALPINE_LIGHT, AMOLED_BLACK, ROSE_ASH)
