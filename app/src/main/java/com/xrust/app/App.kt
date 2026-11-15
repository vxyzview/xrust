package com.xrust.app

import android.app.Application
import com.xrust.app.nativejni.TerminalNative
import com.xrust.app.util.CrashGuard

class App : Application() {

    override fun onCreate() {
        super.onCreate()
        CrashGuard.install(this)
        TerminalNative.loadLibrary()
    }

    override fun onTerminate() {
        TerminalNative.nativeDestroyAll()
        super.onTerminate()
    }
}
