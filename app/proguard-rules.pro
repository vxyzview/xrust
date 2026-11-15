# Kotlin/Compose defaults are fine; keep JNI symbols.
-keep class com.xrust.app.nativejni.TerminalNative { *; }
-keep class com.xrust.app.nativejni.TerminalNative$* { *; }
-keepattributes Signature, InnerClasses, EnclosingMethod
# kotlinx.serialization
-keep,includedescriptorclasses class com.xrust.app.**$$serializer { *; }
-keepclassmembers class com.xrust.app.** {
    *** Companion;
}
-keepclasseswithmembers class com.xrust.app.** {
    kotlinx.serialization.KSerializer serializer(...);
}
