package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.xrust.app.installer.AlpineInstaller
import com.xrust.app.installer.Releases
import com.xrust.app.ui.responsive.ResponsivePane
import kotlinx.coroutines.launch

/** Screen 3: Alpine rootfs installer wizard. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InstallerScreen(installer: AlpineInstaller, onDone: () -> Unit) {
    var progress by remember { mutableStateOf("idle") }
    var percent by remember { mutableStateOf(0) }
    var error by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val abi = remember { android.os.Build.SUPPORTED_ABIS.firstOrNull() ?: "arm64-v8a" }

    Column(Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Alpine installer") })
        // Wizard stays readable and reachable on any window: centered, capped
        // at 480dp, and scrollable when landscape height is tight.
        ResponsivePane(maxContentWidth = 480.dp) {
            Column(
                Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(20.dp),
            ) {
                Text(
                "Installs the official Alpine ${Releases.forAbi(abi).version} " +
                    "minirootfs ($abi) into app-private storage. The archive is " +
                    "SHA-256-verified against dl-cdn.alpinelinux.org before extraction.",
                style = MaterialTheme.typography.bodyMedium,
            )
            if (installer.isInstalled) {
                Text(
                    "Installed: ${installer.installedVersion()}",
                    Modifier.padding(top = 8.dp),
                    color = MaterialTheme.colorScheme.primary,
                )
            }
            if (busy) {
                LinearProgressIndicator(
                    progress = { percent / 100f },
                    Modifier.fillMaxWidth().padding(top = 16.dp),
                )
            }
            Text(progress, Modifier.padding(top = 8.dp), style = MaterialTheme.typography.labelMedium)
            error?.let {
                Card(
                    Modifier.fillMaxWidth().padding(top = 12.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer,
                    ),
                ) {
                    Text(
                        it,
                        Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onErrorContainer,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
            Button(
                onClick = {
                    busy = true; error = null; percent = 0
                    scope.launch {
                        try {
                            installer.install(abi) { p ->
                                when (p) {
                                    is AlpineInstaller.Progress.Downloading -> { percent = p.percent; progress = "downloading ${p.percent}%" }
                                    is AlpineInstaller.Progress.Verifying -> progress = "verifying ${p.size} bytes…"
                                    is AlpineInstaller.Progress.Extracting -> progress = "extracting ${p.entries} entries…"
                                    is AlpineInstaller.Progress.Done -> progress = "done"
                                    is AlpineInstaller.Progress.Failed -> { error = p.message; progress = "failed" }
                                }
                            }
                            installer.writeProfileInit()
                        } catch (e: Exception) {
                            error = e.message
                        } finally {
                            busy = false
                        }
                    }
                },
                Modifier.padding(top = 16.dp),
                enabled = !busy,
            ) { Text(if (installer.isInstalled) "Reinstall" else "Install") }
            if (installer.isInstalled) {
                OutlinedButton(
                    onClick = {
                        installer.rootfsDir.deleteRecursively()
                        progress = "removed"
                    },
                    Modifier.padding(top = 8.dp),
                ) { Text("Remove rootfs") }
            }
            OutlinedButton(onClick = onDone, Modifier.padding(top = 8.dp)) { Text("Back") }
            }
        }
    }
}
