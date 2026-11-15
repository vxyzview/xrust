package com.xrust.app.ui.screens

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.xrust.app.installer.AlpineInstaller
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 1: session list / home. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SessionsScreen(
    installer: AlpineInstaller,
    onOpenTerminal: (String) -> Unit,
    onInstaller: () -> Unit,
) {
    var installed by remember { mutableStateOf(installer.isInstalled) }
    var version by remember { mutableStateOf(installer.installedVersion()) }

    Column(Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("xrust") })
        // Cap the content width on tablets so cards do not stretch the
        // whole window; compact phones fill it exactly as before.
        ResponsivePane(maxContentWidth = 560.dp) {
            LazyColumn(
                Modifier.fillMaxSize().padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
            item {
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant,
                    ),
                ) {
                    Column(Modifier.fillMaxWidth().padding(16.dp)) {
                        Text("Alpine rootfs", style = MaterialTheme.typography.titleMedium)
                        Text(
                            if (installed) "Installed: Alpine $version"
                            else "Not installed — set it up to unlock the full shell",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                        Row(
                            Modifier.fillMaxWidth()
                                .horizontalScroll(rememberScrollState())
                                .padding(top = 8.dp),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            Button(onClick = onInstaller) {
                                Text(if (installed) "Manage" else "Install")
                            }
                            if (installed) {
                                Button(onClick = { onOpenTerminal("alpine-default") }) { Text("Open shell") }
                            }
                        }
                    }
                }
            }
            item {
                Card {
                    Column(Modifier.padding(16.dp)) {
                        Text("Host shell (restricted)", style = MaterialTheme.typography.titleMedium)
                        Text(
                            "Run Android's ToyBox sh directly. Limited by the OS sandbox.",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                        // The host shell needs no Alpine rootfs; it opens the
                        // host profile directly (a rootfs-less alpine profile
                        // would throw in ProotLauncher).
                        Button(onClick = { onOpenTerminal("host-default") }, Modifier.padding(top = 8.dp)) { Text("Open") }
                    }
                }
            }
            }
        }
    }
}
