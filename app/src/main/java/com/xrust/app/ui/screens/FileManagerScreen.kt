package com.xrust.app.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.InsertDriveFile
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.xrust.app.files.FileManager
import com.xrust.app.ui.responsive.ResponsivePane
import java.io.File

/** Screen 4: dual-side file manager (app storage ↔ Alpine rootfs). */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FileManagerScreen() {
    val context = LocalContext.current
    val manager = remember(context) {
        FileManager { side ->
            when (side) {
                "android" -> File(context.filesDir, "share").apply { mkdirs() }
                else -> File(context.filesDir, "alpine").apply { mkdirs() }
            }
        }
    }
    var side by remember { mutableStateOf("android") }
    var cwd by remember { mutableStateOf<String?>(null) }
    val rootPath = remember(side) {
        when (side) {
            "android" -> File(context.filesDir, "share").absolutePath
            else -> File(context.filesDir, "alpine").absolutePath
        }
    }

    Column(Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text("Files — ${if (side == "android") "Android" else "Alpine rootfs"}") },
            navigationIcon = {
                IconButton(onClick = {
                    cwd = cwd?.let { File(it).parentFile?.absolutePath?.takeIf { p -> p.startsWith(rootPath) } }
                }) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "up") }
            },
        )
        Row(Modifier.padding(8.dp)) {
            FilterChip(
                selected = side == "android",
                onClick = { side = "android"; cwd = null },
                label = { Text("Android") },
            )
            FilterChip(
                selected = side != "android",
                onClick = { side = "alpine"; cwd = null },
                label = { Text("Alpine") },
                modifier = Modifier.padding(start = 8.dp),
            )
        }
        HorizontalDivider()
        val entries = manager.list(side, cwd ?: rootPath)
        // Cap list width on wide windows; rows keep a 48dp minimum touch height.
        ResponsivePane(maxContentWidth = 640.dp) {
            LazyColumn(Modifier.fillMaxSize()) {
                items(entries, key = { it.path }) { e ->
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .heightIn(min = 48.dp)
                            .padding(horizontal = 12.dp, vertical = 6.dp),
                    ) {
                        Icon(if (e.isDir) Icons.Filled.Folder else Icons.Filled.InsertDriveFile, null)
                        Column(Modifier.padding(start = 10.dp).weight(1f)) {
                            Text(e.name, style = MaterialTheme.typography.bodyLarge)
                            Text(
                                if (e.isDir) "directory" else "${e.size} bytes",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        if (!e.isDir) {
                            IconButton(onClick = { manager.delete(e.path) }) {
                                Icon(Icons.Filled.Delete, "delete")
                            }
                        }
                    }
                }
            }
        }
    }
}
