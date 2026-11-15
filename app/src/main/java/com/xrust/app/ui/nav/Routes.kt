package com.xrust.app.ui.nav

/** The 12 screens of the app. */
object Routes {
    const val SESSIONS = "sessions"
    const val TERMINAL = "terminal/{profileId}"
    const val INSTALLER = "installer"
    const val FILES = "files"
    const val SETTINGS = "settings"
    const val DISPLAY = "display"
    const val FONT = "font"
    const val SCHEMES = "schemes"
    const val KEYBOARD = "keyboard"
    const val PROFILES = "profiles"
    const val PROFILE_EDIT = "profile/{profileId}"
    const val ABOUT = "about"

    fun terminal(profileId: String) = "terminal/$profileId"
    fun profileEdit(profileId: String) = "profile/$profileId"
}
