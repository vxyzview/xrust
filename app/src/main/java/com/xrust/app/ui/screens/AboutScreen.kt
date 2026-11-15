package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 12: about / help. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AboutScreen() {
    ResponsivePane(maxContentWidth = 600.dp) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
        TopAppBar(title = { Text("About") })
        Column(Modifier.padding(16.dp)) {
            Text("xrust 1.0.0", style = MaterialTheme.typography.titleLarge)
            Text(
                "A Rust-first terminal for Android: the VT parser, grid, " +
                    "scrollback, reflow, selection and PTY live in native code " +
                    "(xrust-core), rendered by Jetpack Compose Material 3. " +
                    "Alpine userland runs through PRoot v5.5.0.",
                Modifier.padding(top = 8.dp),
            )
            HorizontalDivider(Modifier.padding(vertical = 12.dp))
            Text("Architecture", style = MaterialTheme.typography.titleMedium)
            Text(
                "• xrust-core (Rust): vte parser, grid, scrollback, reflow, " +
                    "keymap, PTY, JNI bridge — 134 unit/integration tests.\n" +
                    "• app (Kotlin/Compose): 12 screens, DataStore settings, " +
                    "profile JSON store, file manager, installer with SHA-256 " +
                    "verification.\n• PRoot 5.5.0 per-ABI binaries built in CI.",
                Modifier.padding(top = 4.dp),
            )
            HorizontalDivider(Modifier.padding(vertical = 12.dp))
            Text("Host shell limits", style = MaterialTheme.typography.titleMedium)
            Text(
                "Host sessions run Android's ToyBox sh inside the app sandbox: " +
                    "no package manager, no chroot, only app-visible paths. Use " +
                    "the Alpine rootfs for a complete userland.",
                Modifier.padding(top = 4.dp),
            )
            HorizontalDivider(Modifier.padding(vertical = 12.dp))
            Text("Profile JSON schema", style = MaterialTheme.typography.titleMedium)
            Text(
                """[{"id":"x","name":"Alpine sh","kind":"alpine",
                   |"command":"/bin/sh","args":["-l"],
                   |"env":{"KEY":"V"},"cwd":"/root",
                   |"colorScheme":"Alpine Dark","fontSizeSp":13,
                   |"extraKeysLayout":"default"}]""".trimMargin(),
                Modifier.padding(top = 4.dp),
            )
            HorizontalDivider(Modifier.padding(vertical = 12.dp))
            Text("License", style = MaterialTheme.typography.titleMedium)
            Text("MIT — see LICENSE in the repository.", Modifier.padding(top = 4.dp))
        }
    }
    }
}
