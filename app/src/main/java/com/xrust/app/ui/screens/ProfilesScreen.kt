package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
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
import com.xrust.app.ui.nav.Routes

/** Screen 10: profile list with import/export. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfilesScreen(profilesStore: ProfilesStore, nav: NavHostController) {
    var profiles by remember { mutableStateOf(profilesStore.load()) }

    Scaffold(
        floatingActionButton = {
            FloatingActionButton(onClick = {
                val p = Profile(
                    id = "p-${System.currentTimeMillis()}",
                    name = "New profile",
                )
                profilesStore.save(profiles + p)
                profiles = profilesStore.load()
                nav.navigate(Routes.profileEdit(p.id))
            }) { Icon(Icons.Filled.Add, "add") }
        },
    ) { padding ->
        LazyColumn(Modifier.fillMaxSize().padding(padding)) {
            item { TopAppBar(title = { Text("Profiles") }) }
            items(profiles, key = { it.id }) { p ->
                Card(
                    Modifier.fillMaxWidth().padding(12.dp, 6.dp),
                    onClick = { nav.navigate(Routes.profileEdit(p.id)) },
                ) {
                    Column(Modifier.padding(14.dp)) {
                        Text(p.name, style = MaterialTheme.typography.titleMedium)
                        Text(
                            "${p.kind} · ${p.command} ${p.args.joinToString(" ")} · cwd ${p.cwd}",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
            }
            item {
                Text(
                    "Import/export uses JSON (see About for the schema).",
                    Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
