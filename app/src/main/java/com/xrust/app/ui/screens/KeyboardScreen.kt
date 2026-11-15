package com.xrust.app.ui.screens

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
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
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 9: keyboard & input behavior. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun KeyboardScreen(settingsRepo: SettingsRepository, nav: NavHostController) {
    val s by settingsRepo.settings.collectAsState(initial = TerminalSettings())
    // Scrollable: the ListItem stack overflows landscape heights otherwise.
    ResponsivePane(maxContentWidth = 600.dp) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
        TopAppBar(title = { Text("Keyboard & input") })
        Text("Extra-keys layout", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        Row(Modifier.horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp)) {
            listOf("default", "vi", "emacs", "none").forEach { layout ->
                FilterChip(
                    selected = s.extraKeysLayout == layout,
                    onClick = { settingsRepo.set { it.copy(extraKeysLayout = layout) } },
                    label = { Text(layout) },
                    modifier = Modifier.padding(end = 8.dp),
                )
            }
        }
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Backspace sends DEL (\\x7f)") },
            supportingContent = { Text("Turn off to send \\x08 instead (some ancient programs)") },
            trailingContent = { Switch(checked = s.backspaceIsDelete, onCheckedChange = { v -> settingsRepo.set { it.copy(backspaceIsDelete = v) } }) },
        )
        ListItem(
            headlineContent = { Text("Alt sends ESC prefix") },
            supportingContent = { Text("Meta key behavior for external keyboards") },
            trailingContent = { Switch(checked = s.altAsEsc, onCheckedChange = { v -> settingsRepo.set { it.copy(altAsEsc = v) } }) },
        )
        ListItem(
            headlineContent = { Text("Bracketed paste") },
            supportingContent = { Text("Paste wraps in \\e[200~ … \\e[201~ when the shell enables it") },
            trailingContent = { Switch(checked = s.bracketedPaste, onCheckedChange = { v -> settingsRepo.set { it.copy(bracketedPaste = v) } }) },
        )
        ListItem(
            headlineContent = { Text("Haptic feedback on keys") },
            trailingContent = { Switch(checked = s.hapticFeedback, onCheckedChange = { v -> settingsRepo.set { it.copy(hapticFeedback = v) } }) },
        )
        ListItem(
            headlineContent = { Text("Invert scroll direction") },
            trailingContent = { Switch(checked = s.invertScrollHistory, onCheckedChange = { v -> settingsRepo.set { it.copy(invertScrollHistory = v) } }) },
        )
        HorizontalDivider()
        Text("Default grid size", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        Text("${s.defaultCols} × ${s.defaultRows}", Modifier.padding(16.dp))
    }
    }
}
