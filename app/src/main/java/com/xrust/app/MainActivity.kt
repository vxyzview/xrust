package com.xrust.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Terminal
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.lifecycleScope
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.xrust.app.data.Profile
import com.xrust.app.data.ProfilesStore
import com.xrust.app.data.SessionMeta
import com.xrust.app.data.SettingsRepository
import com.xrust.app.data.SessionsStore
import com.xrust.app.data.TerminalSettings
import com.xrust.app.installer.AlpineInstaller
import com.xrust.app.nativejni.TerminalNative
import com.xrust.app.proot.ProotLauncher
import com.xrust.app.terminal.TerminalController
import com.xrust.app.ui.nav.Routes
import com.xrust.app.ui.screens.AboutScreen
import com.xrust.app.ui.screens.ColorSchemeScreen
import com.xrust.app.ui.screens.DisplaySettingsScreen
import com.xrust.app.ui.screens.FileManagerScreen
import com.xrust.app.ui.screens.FontScreen
import com.xrust.app.ui.screens.InstallerScreen
import com.xrust.app.ui.screens.KeyboardScreen
import com.xrust.app.ui.screens.ProfileEditScreen
import com.xrust.app.ui.screens.ProfilesScreen
import com.xrust.app.ui.screens.SessionsScreen
import com.xrust.app.ui.screens.SettingsScreen
import com.xrust.app.ui.screens.TerminalScreen
import com.xrust.app.ui.theme.xrustTheme
import kotlinx.coroutines.launch
import java.io.File

class MainActivity : ComponentActivity() {

    private val settingsRepo by lazy { SettingsRepository(this) }
    private val profilesStore by lazy { ProfilesStore { File(filesDir, "profiles.json") } }
    private val sessionsFile by lazy { File(filesDir, "sessions.json") }
    private val installer by lazy { AlpineInstaller(this) }
    private val controllers = mutableMapOf<Long, TerminalController>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val saved = SessionsStore.load(sessionsFile)
        setContent {
            val settings by settingsRepo.settings.collectAsState(initial = TerminalSettings())
            xrustTheme(mode = settings.themeMode) {
                AppRoot(
                    installer = installer,
                    profilesStore = profilesStore,
                    settingsRepo = settingsRepo,
                    getController = { profileId -> obtainController(profileId, settings) },
                    closeController = { id -> controllers.remove(id)?.destroy(); persistSessions(controllers.keys.toList()) },
                    persistSessions = { ids -> persistSessions(ids) },
                )
            }
        }
    }

    private fun obtainController(profileId: String, settings: TerminalSettings): TerminalController {
        val existing = controllers.values.firstOrNull()
        if (existing != null) return existing
        val profile: Profile = profilesStore.load().firstOrNull { it.id == profileId }
            ?: profilesStore.defaultProfiles().first()
        val id = TerminalNative.nativeCreate(settings.defaultCols, settings.defaultRows, settings.scrollbackLines)
        val controller = TerminalController(id, settings.defaultCols, settings.defaultRows, settings.scrollbackLines)
        val argv = ProotLauncher.argv(this, profile)
        val env = ProotLauncher.env(profile).toTypedArray()
        val ok = TerminalNative.nativeSpawn(id, argv.first(), argv.drop(1).toTypedArray(), env, profile.cwd)
        check(ok) { "failed to spawn ${argv.first()}" }
        installer.writeProfileInit()
        controller.setListener(object : TerminalNative.Listener {
            override fun onTitle(title: String) { /* controller observes via refresh */ }
            override fun onBell() { /* bell handled in UI via controller.bellTick */ }
            override fun onExit(code: Int) { lifecycleScope.launch { persistSessions(controllers.keys.toList()) } }
            override fun onCwd(cwd: String) { /* controller exposes cwd */ }
        })
        controllers[id] = controller
        persistSessions(listOf(id))
        return controller
    }

    private fun persistSessions(ids: List<Long>) {
        runCatching {
            SessionsStore.save(
                sessionsFile,
                ids.map { SessionMeta(it, "alpine-default", "xrust sh", System.currentTimeMillis()) },
            )
        }
    }

    override fun onDestroy() {
        controllers.forEach { (_, c) -> c.destroy() }
        controllers.clear()
        super.onDestroy()
    }
}

@Composable
fun AppRoot(
    installer: AlpineInstaller,
    profilesStore: ProfilesStore,
    settingsRepo: SettingsRepository,
    getController: (String) -> TerminalController,
    closeController: (Long) -> Unit,
    persistSessions: (List<Long>) -> Unit,
) {
    val nav = rememberNavController()
    val backStack by nav.currentBackStackEntryAsState()
    val route = backStack?.destination?.route ?: Routes.SESSIONS
    val showBottomBar = route in setOf(Routes.SESSIONS, Routes.FILES, Routes.SETTINGS, Routes.PROFILES, Routes.ABOUT)

    Scaffold(
        bottomBar = {
            if (showBottomBar) {
                NavigationBar {
                    NavigationBarItem(
                        selected = route == Routes.SESSIONS,
                        onClick = { nav.navigate(Routes.SESSIONS) { popUpTo(Routes.SESSIONS) { saveState = true } } },
                        icon = { Icon(Icons.Filled.Terminal, null) },
                        label = { Text("Terminals") },
                    )
                    NavigationBarItem(
                        selected = route == Routes.FILES,
                        onClick = { nav.navigate(Routes.FILES) { popUpTo(Routes.SESSIONS) { saveState = true } } },
                        icon = { Icon(Icons.Filled.Description, null) },
                        label = { Text("Files") },
                    )
                    NavigationBarItem(
                        selected = route == Routes.PROFILES,
                        onClick = { nav.navigate(Routes.PROFILES) { popUpTo(Routes.SESSIONS) { saveState = true } } },
                        icon = { Icon(Icons.Filled.Person, null) },
                        label = { Text("Profiles") },
                    )
                    NavigationBarItem(
                        selected = route == Routes.SETTINGS,
                        onClick = { nav.navigate(Routes.SETTINGS) { popUpTo(Routes.SESSIONS) { saveState = true } } },
                        icon = { Icon(Icons.Filled.Settings, null) },
                        label = { Text("Settings") },
                    )
                    NavigationBarItem(
                        selected = route == Routes.ABOUT,
                        onClick = { nav.navigate(Routes.ABOUT) { popUpTo(Routes.SESSIONS) { saveState = true } } },
                        icon = { Icon(Icons.Filled.Info, null) },
                        label = { Text("About") },
                    )
                }
            }
        },
    ) { padding ->
        NavHost(nav, startDestination = Routes.SESSIONS, modifier = Modifier.padding(padding)) {
            composable(Routes.SESSIONS) {
                SessionsScreen(
                    installer = installer,
                    onOpenTerminal = { nav.navigate(Routes.terminal(0)) },
                    onInstaller = { nav.navigate(Routes.INSTALLER) },
                )
            }
            composable(
                Routes.TERMINAL,
                arguments = listOf(navArgument("sessionId") { type = NavType.LongType }),
            ) { _ ->
                TerminalScreen(
                    getController = { getController("alpine-default") },
                    onClose = { closeController(0); nav.popBackStack() },
                )
            }
            composable(Routes.INSTALLER) { InstallerScreen(installer = installer, onDone = { nav.popBackStack() }) }
            composable(Routes.FILES) { FileManagerScreen() }
            composable(Routes.SETTINGS) { SettingsScreen(settingsRepo = settingsRepo, nav = nav) }
            composable(Routes.DISPLAY) { DisplaySettingsScreen(settingsRepo = settingsRepo, nav = nav) }
            composable(Routes.FONT) { FontScreen(settingsRepo = settingsRepo, nav = nav) }
            composable(Routes.SCHEMES) { ColorSchemeScreen(settingsRepo = settingsRepo, nav = nav) }
            composable(Routes.KEYBOARD) { KeyboardScreen(settingsRepo = settingsRepo, nav = nav) }
            composable(Routes.PROFILES) { ProfilesScreen(profilesStore = profilesStore, nav = nav) }
            composable(
                Routes.PROFILE_EDIT,
                arguments = listOf(navArgument("profileId") { type = NavType.StringType }),
            ) { entry ->
                ProfileEditScreen(
                    profilesStore = profilesStore,
                    profileId = entry.arguments?.getString("profileId") ?: "",
                    nav = nav,
                )
            }
            composable(Routes.ABOUT) { AboutScreen() }
        }
    }
}
