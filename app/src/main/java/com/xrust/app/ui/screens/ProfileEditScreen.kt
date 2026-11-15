package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import com.xrust.app.data.Profile
import com.xrust.app.data.ProfilesStore
import com.xrust.app.ui.responsive.ResponsivePane

/** Screen 11: profile editor. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfileEditScreen(profilesStore: ProfilesStore, profileId: String, nav: NavHostController) {
    var profiles by remember { mutableStateOf(profilesStore.load()) }
    var p by remember(profileId) {
        mutableStateOf(profiles.firstOrNull { it.id == profileId } ?: Profile(id = profileId, name = "New profile"))
    }
    var error by remember { mutableStateOf<String?>(null) }

    fun update(transform: (Profile) -> Profile) {
        p = transform(p)
        error = null
    }

    // Form fields stay near their natural width on any window.
    ResponsivePane(maxContentWidth = 600.dp) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
        TopAppBar(title = { Text("Edit profile") })
        OutlinedTextField(
            value = p.name,
            onValueChange = { newName -> update { it.copy(name = newName) } },
            label = { Text("Name") },
            Modifier.fillMaxWidth().padding(12.dp, 6.dp),
        )
        Row(Modifier.padding(12.dp, 4.dp)) {
            listOf("alpine", "host").forEach { kind ->
                FilterChip(
                    selected = p.kind == kind,
                    onClick = { update { it.copy(kind = kind) } },
                    label = { Text(kind) },
                    modifier = Modifier.padding(end = 8.dp),
                )
            }
        }
        OutlinedTextField(
            value = p.command,
            onValueChange = { cmd -> update { it.copy(command = cmd) } },
            label = { Text("Command") },
            Modifier.fillMaxWidth().padding(12.dp, 6.dp),
        )
        OutlinedTextField(
            value = p.args.joinToString(" "),
            onValueChange = { a -> update { it.copy(args = a.split(" ").filter { t -> t.isNotBlank() }) } },
            label = { Text("Arguments (space separated)") },
            Modifier.fillMaxWidth().padding(12.dp, 6.dp),
        )
        OutlinedTextField(
            value = p.cwd,
            onValueChange = { c -> update { it.copy(cwd = c) } },
            label = { Text("Working directory") },
            Modifier.fillMaxWidth().padding(12.dp, 6.dp),
        )
        OutlinedTextField(
            value = p.env.entries.joinToString("\n") { "${it.key}=${it.value}" },
            onValueChange = { txt ->
                update { cur ->
                    cur.copy(
                        env = txt.lines().filter { it.contains('=') }.associate {
                            val i = it.indexOf('=')
                            it.substring(0, i).trim() to it.substring(i + 1)
                        },
                    )
                }
            },
            label = { Text("Environment (KEY=VALUE per line)") },
            Modifier.fillMaxWidth().padding(12.dp, 6.dp),
        )
        HorizontalDivider(Modifier.padding(12.dp))
        Button(
            onClick = {
                try {
                    profilesStore.save(profiles.filter { it.id != p.id } + p)
                    nav.popBackStack()
                } catch (e: Exception) {
                    error = e.message
                }
            },
            Modifier.padding(12.dp),
        ) { Text("Save") }
        Button(
            onClick = {
                profilesStore.save(profiles.filter { it.id != p.id })
                nav.popBackStack()
            },
            Modifier.padding(12.dp, 0.dp),
        ) { Text("Delete") }
        error?.let { Text(it, Modifier.padding(12.dp), color = MaterialTheme.colorScheme.error) }
    }
    }
}
