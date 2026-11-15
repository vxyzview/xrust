package com.xrust.app.ui.responsive

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * Manual window-width classes (Material 3 breakpoints) — no extra artifact
 * needed, so the build stays resolvable fully offline.
 */
enum class WindowWidth { Compact, Medium, Expanded }

@Composable
fun rememberWindowWidth(): WindowWidth {
    val widthDp = LocalConfiguration.current.screenWidthDp
    return when {
        widthDp < 600 -> WindowWidth.Compact
        widthDp < 840 -> WindowWidth.Medium
        else -> WindowWidth.Expanded
    }
}

/** True when the window is wider than it is tall (phone landscape, split screen). */
@Composable
fun rememberIsLandscape(): Boolean {
    val cfg = LocalConfiguration.current
    return cfg.screenWidthDp > cfg.screenHeightDp
}

/**
 * Centers content and caps its width on medium/expanded windows so lists and
 * text do not stretch across a tablet. On compact phones the pane fills the
 * window exactly as before.
 */
@Composable
fun ResponsivePane(
    modifier: Modifier = Modifier,
    maxContentWidth: Dp = 640.dp,
    content: @Composable BoxScope.() -> Unit,
) {
    Box(modifier.fillMaxSize()) {
        Box(
            Modifier
                .align(Alignment.TopCenter)
                .fillMaxSize()
                .widthIn(max = maxContentWidth),
            content = content,
        )
    }
}
