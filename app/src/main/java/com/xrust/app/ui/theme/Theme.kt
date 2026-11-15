package com.xrust.app.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

/** App theme modes (settings). */
enum class ThemeMode { SYSTEM, ALPINE_DARK, ALPINE_LIGHT, AMOLED, ROSE_ASH }

@Composable
fun xrustTheme(
    mode: ThemeMode,
    content: @Composable () -> Unit,
) {
    val dark = isSystemInDarkTheme()
    // Material You: wallpaper-derived dynamic color on Android 12+ in SYSTEM
    // mode; every other mode uses its own complete tonal scheme.
    if (mode == ThemeMode.SYSTEM && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val ctx = LocalContext.current
        val dyn = if (dark) dynamicDarkColorScheme(ctx) else dynamicLightColorScheme(ctx)
        MaterialTheme(colorScheme = dyn, typography = AppTypography, content = content)
    } else {
        val scheme = when (mode) {
            ThemeMode.SYSTEM -> if (dark) ALPINE_DARK else ALPINE_LIGHT
            ThemeMode.ALPINE_DARK -> ALPINE_DARK
            ThemeMode.ALPINE_LIGHT -> ALPINE_LIGHT
            ThemeMode.AMOLED -> AMOLED_BLACK
            ThemeMode.ROSE_ASH -> ROSE_ASH
        }
        MaterialTheme(colorScheme = scheme.scheme, typography = AppTypography, content = content)
    }
}
