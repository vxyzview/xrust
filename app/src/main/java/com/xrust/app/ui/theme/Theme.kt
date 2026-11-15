package com.xrust.app.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

/** App theme modes (settings). */
enum class ThemeMode { SYSTEM, ALPINE_DARK, ALPINE_LIGHT, AMOLED, ROSE_ASH }

@Composable
fun XrustTheme(
    mode: ThemeMode,
    content: @Composable () -> Unit,
) {
    val dark = isSystemInDarkTheme()
    val colors = when (mode) {
        ThemeMode.SYSTEM -> if (dark) ALPINE_DARK else ALPINE_LIGHT
        ThemeMode.ALPINE_DARK -> ALPINE_DARK
        ThemeMode.ALPINE_LIGHT -> ALPINE_LIGHT
        ThemeMode.AMOLED -> AMOLED_BLACK
        ThemeMode.ROSE_ASH -> ROSE_ASH
    }
    val scheme = if (colors.isDark) {
        darkColorScheme(
            primary = colors.primary,
            background = colors.background,
            surface = colors.surface,
            surfaceVariant = colors.surfaceVariant,
        )
    } else {
        lightColorScheme(
            primary = colors.primary,
            background = colors.background,
            surface = colors.surface,
            surfaceVariant = colors.surfaceVariant,
        )
    }
    // Dynamic color (Material You) is offered only in SYSTEM mode on S+.
    val useDynamic = mode == ThemeMode.SYSTEM && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S
    if (useDynamic) {
        val ctx = LocalContext.current
        val dyn = if (dark) dynamicDarkColorScheme(ctx) else dynamicLightColorScheme(ctx)
        MaterialTheme(colorScheme = dyn, typography = AppTypography, content = content)
    } else {
        MaterialTheme(colorScheme = scheme, typography = AppTypography, content = content)
    }
}
