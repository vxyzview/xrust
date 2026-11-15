package com.xrust.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import com.xrust.app.data.SettingsRepository
import com.xrust.app.data.TerminalSettings
import com.xrust.app.ui.theme.ALL_TERMINAL_SCHEMES

/** Screen 8: terminal color scheme picker with live palette preview. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ColorSchemeScreen(settingsRepo: SettingsRepository, nav: NavHostController) {
    val s by settingsRepo.settings.collectAsState(initial = TerminalSettings())
    LazyColumn(Modifier.fillMaxSize().padding(bottom = 16.dp)) {
        item { TopAppBar(title = { Text("Color schemes") }) }
        items(ALL_TERMINAL_SCHEMES, key = { it.name }) { scheme ->
            Card(
                Modifier.fillMaxWidth().padding(12.dp, 6.dp),
                shape = RoundedCornerShape(10.dp),
                onClick = { settingsRepo.set { it.copy(colorScheme = scheme.name) } },
            ) {
                Column(Modifier.padding(12.dp)) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        listOf(scheme.background, scheme.foreground, scheme.cursor).forEach { c ->
                            Box(
                                Modifier.height(22.dp).weight(1f).background(
                                    Color(c or 0xFF000000u.toInt()),
                                    RoundedCornerShape(4.dp),
                                ),
                            )
                        }
                        scheme.palette.take(8).forEach { c ->
                            Box(
                                Modifier.height(22.dp).weight(1f).background(
                                    Color(c or 0xFF000000u.toInt()),
                                    RoundedCornerShape(4.dp),
                                ),
                            )
                        }
                    }
                    Row(Modifier.fillMaxWidth().padding(top = 6.dp)) {
                        Text(scheme.name, style = MaterialTheme.typography.titleMedium)
                        Text(
                            if (scheme.name == s.colorScheme) "  • in use" else "",
                            color = MaterialTheme.colorScheme.primary,
                        )
                        Text(
                            if (scheme.isDark) "  dark" else "  light",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }
    }
}
