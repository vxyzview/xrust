package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import com.xrust.app.data.SettingsRepository
import com.xrust.app.data.TerminalSettings
import com.xrust.app.ui.theme.ALPINE_DARK
import com.xrust.app.ui.theme.ALPINE_LIGHT
import com.xrust.app.ui.theme.AMOLED_BLACK
import com.xrust.app.ui.theme.ALL_APP_THEMES
import com.xrust.app.ui.theme.ROSE_ASH
import com.xrust.app.ui.theme.ThemeMode
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 6: display, theme and cursor settings. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DisplaySettingsScreen(settingsRepo: SettingsRepository, nav: NavHostController) {
    val s by settingsRepo.settings.collectAsState(initial = TerminalSettings())
    ResponsivePane(maxContentWidth = 600.dp) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
        TopAppBar(title = { Text("Display & colors") })
        Text("Theme", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        ALL_APP_THEMES.forEach { t ->
            val mode = when (t) {
                ALPINE_DARK -> ThemeMode.ALPINE_DARK
                ALPINE_LIGHT -> ThemeMode.ALPINE_LIGHT
                AMOLED_BLACK -> ThemeMode.AMOLED
                ROSE_ASH -> ThemeMode.ROSE_ASH
                else -> ThemeMode.SYSTEM
            }
            ListItem(
                headlineContent = { Text(t.name) },
                trailingContent = {
                    Switch(checked = s.themeMode == mode, onCheckedChange = { v ->
                        if (v) settingsRepo.set { it.copy(themeMode = mode) }
                    })
                },
            )
        }
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("AMOLED pure black surfaces") },
            supportingContent = { Text("Forces true #000000 background (no elevation tint)") },
            trailingContent = { Switch(checked = s.amoledPureBlack, onCheckedChange = { v -> settingsRepo.set { it.copy(amoledPureBlack = v) } }) },
        )
        HorizontalDivider()
        Text("Cursor", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        ListItem(
            headlineContent = { Text("Cursor blink") },
            trailingContent = { Switch(checked = s.cursorBlink, onCheckedChange = { v -> settingsRepo.set { it.copy(cursorBlink = v) } }) },
        )
        Text("Cursor shape: ${cursorShapeName(s.cursorShape)}", Modifier.padding(16.dp))
        Slider(
            value = s.cursorShape.toFloat(),
            onValueChange = { v -> settingsRepo.set { it.copy(cursorShape = v.toInt()) } },
            valueRange = 0f..2f,
            steps = 1,
        )
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Bell flash") },
            trailingContent = { Switch(checked = s.bellFlash, onCheckedChange = { v -> settingsRepo.set { it.copy(bellFlash = v) } }) },
        )
    }
    }
}

fun cursorShapeName(id: Int) = when (id) {
    0 -> "block"
    1 -> "bar"
    else -> "underline"
}
