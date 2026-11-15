package com.xrust.app.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
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
import com.xrust.app.ui.nav.Routes
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 5: settings hub. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(settingsRepo: SettingsRepository, nav: NavHostController) {
    val s by settingsRepo.settings.collectAsState(initial = TerminalSettings())
    ResponsivePane(maxContentWidth = 600.dp) {
        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
            TopAppBar(title = { Text("Settings") })
        ListItem(
            headlineContent = { Text("Display & colors") },
            supportingContent = { Text("Theme, AMOLED pure black, cursor") },
            modifier = Modifier.clickable { nav.navigate(Routes.DISPLAY) },
        )
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Font") },
            supportingContent = { Text("Family, size ${s.fontSizeSp}sp, line height ${s.lineHeightPct}%") },
            modifier = Modifier.clickable { nav.navigate(Routes.FONT) },
        )
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Color schemes") },
            supportingContent = { Text(s.colorScheme) },
            modifier = Modifier.clickable { nav.navigate(Routes.SCHEMES) },
        )
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Keyboard & input") },
            supportingContent = { Text("Extra keys, backspace, modifiers") },
            modifier = Modifier.clickable { nav.navigate(Routes.KEYBOARD) },
        )
        HorizontalDivider()
        SwitchRow("Vibrate on bell", s.bellVibrate) { v -> settingsRepo.set { it.copy(bellVibrate = v) } }
        SwitchRow("Keep screen on", s.keepScreenOn) { v -> settingsRepo.set { it.copy(keepScreenOn = v) } }
        SwitchRow("Show extra-keys toolbar", s.extraKeysVisible) { v -> settingsRepo.set { it.copy(extraKeysVisible = v) } }
        Text(
            "Terminal parameters exposed: 25 (theme, font, cursor, scrollback, " +
                "bells, margins, input modes, layouts…)",
            Modifier.padding(16.dp),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        }
    }
}

@Composable
fun SwitchRow(label: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    ListItem(
        headlineContent = { Text(label) },
        trailingContent = { Switch(checked = checked, onCheckedChange = onChange) },
    )
}
