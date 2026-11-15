package com.xrust.app.ui.screens

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavHostController
import com.xrust.app.data.SettingsRepository
import com.xrust.app.data.TerminalSettings

/** Screen 7: font family/size/spacing. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FontScreen(settingsRepo: SettingsRepository, nav: NavHostController) {
    val s by settingsRepo.settings.collectAsState(initial = TerminalSettings())
    Column(Modifier.fillMaxSize().padding(bottom = 24.dp)) {
        TopAppBar(title = { Text("Font") })
        Text("Family", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        Row(Modifier.horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp)) {
            listOf("System mono", "Serif mono", "Compact").forEachIndexed { i, name ->
                FilterChip(
                    selected = s.fontFamilyId == i,
                    onClick = { settingsRepo.set { it.copy(fontFamilyId = i) } },
                    label = { Text(name) },
                    modifier = Modifier.padding(end = 8.dp),
                )
            }
        }
        HorizontalDivider()
        Text("Size: ${s.fontSizeSp}sp", Modifier.padding(16.dp, 8.dp))
        Slider(
            value = s.fontSizeSp.toFloat(),
            onValueChange = { v -> settingsRepo.set { it.copy(fontSizeSp = v.toInt()) } },
            valueRange = 8f..24f,
        )
        Text("Line height: ${s.lineHeightPct}%", Modifier.padding(16.dp, 8.dp))
        Slider(
            value = s.lineHeightPct.toFloat(),
            onValueChange = { v -> settingsRepo.set { it.copy(lineHeightPct = v.toInt()) } },
            valueRange = 90f..200f,
        )
        Text("Letter spacing: ${s.letterSpacing / 10f}px", Modifier.padding(16.dp, 8.dp))
        Slider(
            value = s.letterSpacing.toFloat(),
            onValueChange = { v -> settingsRepo.set { it.copy(letterSpacing = v.toInt()) } },
            valueRange = -10f..20f,
        )
        Text("DPI scale: ${s.fontDpiScale}%", Modifier.padding(16.dp, 8.dp))
        Slider(
            value = s.fontDpiScale.toFloat(),
            onValueChange = { v -> settingsRepo.set { it.copy(fontDpiScale = v.toInt()) } },
            valueRange = 50f..200f,
        )
        HorizontalDivider()
        Text("Preview", Modifier.padding(16.dp, 8.dp), style = MaterialTheme.typography.titleMedium)
        Text(
            "xrust:~$ ls -la --color=auto",
            Modifier.padding(16.dp),
            fontSize = (s.fontSizeSp * s.fontDpiScale / 100).sp,
        )
    }
}
