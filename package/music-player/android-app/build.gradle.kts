// Root build file. The Android Gradle Plugin (AGP) 9 runs Kotlin through its built-in Kotlin
// support, so the standalone `org.jetbrains.kotlin.android` plugin is not applied here;
// only the Compose compiler plugin is layered on in :app. AGP 9.4.1 carries a runtime
// dependency on Kotlin Gradle plugin (KGP) 2.2.10, and Gradle upgrades that to any higher
// KGP declared on this build's classpath, so the `buildscript` block below raises the
// Kotlin that compiles the app to the same version as the Compose compiler plugin.
buildscript {
    dependencies {
        // What:     `classpath(...)` adds the Kotlin Gradle plugin to the root build's plugin
        //           classpath, where it outranks the 2.2.10 that AGP requires.
        //           This is the documented way to run built-in Kotlin on a newer KGP.
        // Why:      The Compose compiler plugin below must match the Kotlin compiler version.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // build.pluginClasspath.add("org.jetbrains.kotlin:kotlin-gradle-plugin:2.4.21");
        // ```
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.4.21")
    }
}

plugins {
    id("com.android.application") version "9.4.1" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.21" apply false
    // Checks the license of every library the app ships against an allow-list and lists them for the license
    // notices (doc/decision/music-player-third-party-licenses.md). 1.14.1 is the newest release (2025-10-09); it was
    // run against this build's AGP 9.4.1 and Gradle 9.8.1 before adoption.
    id("app.cash.licensee") version "1.14.1" apply false
}
