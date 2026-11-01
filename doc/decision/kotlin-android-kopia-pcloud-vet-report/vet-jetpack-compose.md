# Vet: raw Jetpack Compose (Android-native androidx.compose) for the kopia + local-S3 + pCloud app

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07.
 Vetter:
 choosing-technology FULL-VERIFICATION standard.
Scope:
 the UI/app stack around the app.
 Running kopia and the pCloud API are out of scope (owner-decided).
Sibling vet covered Compose Multiplatform;
 this covers Android-native Jetpack Compose.

## Verdict (short)

Use Android-native Jetpack Compose.
 It built,
 installed,
 launched,
 rendered,
 and handled
input on a real Pixel 6 on the current 2026 toolchain,
 and the app's hard parts (embedded
HTTP server,
 HTTPS client,
 foreground/background work,
 exec of the kopia Go binary) are all
served by actively maintained Android-first libraries.
 Prefer it over Compose Multiplatform
for this Android-only app:
 CMP only adds a JetBrains abstraction layer and version lag with
no second target to justify it.

## 1. Identity: is "raw Jetpack Compose" Google AOSP androidx.compose?

Yes.
 The artifacts resolve from Google's Maven (`dl.google.com/android/maven2`) under group
`androidx.compose.*` plus the `androidx.compose:compose-bom` platform.
 Source lives in AOSP
(`android.googlesource.com` / `cs.android.com`),
 mirrored read-only to `github.com/androidx/androidx`.
This is distinct from JetBrains' Compose Multiplatform,
 which republishes the same UI API under
`org.jetbrains.compose.*` and adds non-Android targets.

Confirmed by fetching `compose-bom-2026.05.01.pom` (HTTP 200) from Google Maven.

## 2. FULL VERIFICATION: build + run on device

Environment built fresh and isolated under `/var/tmp/vet-jc` (no SDK race with other vets):

- JDK:
   Temurin 21.0.5+11 (JBR 25 on the host is too new for stable Gradle/AGP).
- Gradle 9.5.1,
   Android Gradle Plugin 9.2.1.
- AGP 9.2.1 has built-in Kotlin (KGP 2.2.10);
   the standalone `org.jetbrains.kotlin.android`
  plugin is now rejected.
   Verified by build error,
   then by reading AGP 9.2.1's POM
  (`kotlin-gradle-plugin` -> `2.2.10`).
   The Compose Compiler plugin
  `org.jetbrains.kotlin.plugin.compose` is STILL required and must match Kotlin (2.2.10).
- compileSdk/targetSdk 37 (Android 17),
   minSdk 24,
   build-tools 37.0.0.
- Compose BOM 2026.05.01,
   activity-compose 1.13.0,
   core-ktx 1.19.0.
- Device:
   Pixel 6 (oriole),
   serial 1C171FDF600KWW,
   running GrapheneOS.
   adb serialized via
  `flock /tmp/agent/adb-phone.lock`,
   targeted by `-s <serial>` (other vets may attach emulators).
- applicationId `com.monochromatic.vetjc` (unique).

App:
 a single `ComponentActivity` with `setContent { MaterialTheme { ... } }`,
 a Material3
`Button` + `Text` counter using `remember { mutableIntStateOf(0) }`.

Build:

```bash
gradle-9.5.1/bin/gradle :app:assembleDebug --no-daemon --console=plain
# ... > Task :app:assembleDebug
# BUILD SUCCESSFUL in 52s   (36 tasks)
# APK: app/build/outputs/apk/debug/app-debug.apk  (11.4 MB)
# aapt2 badging: package com.monochromatic.vetjc, compileSdkVersion='37' codename='17',
#                launchable-activity com.monochromatic.vetjc.MainActivity
```

Install + launch + confirm resumed (under flock,
 `-s 1C171FDF600KWW`):

```bash
adb install -r app-debug.apk          # Performing Streamed Install / Success
adb shell am start -n com.monochromatic.vetjc/.MainActivity
adb shell pidof com.monochromatic.vetjc          # 21761
adb shell dumpsys activity activities | grep -i resumed
#   topResumedActivity=ActivityRecord{... com.monochromatic.vetjc/.MainActivity ...}
#   ResumedActivity:   ActivityRecord{... com.monochromatic.vetjc/.MainActivity ...}
#   (visible=true, mode=fullscreen)
```

Render + interaction:
 screencap showed "Count:
 0" with the purple Material3 Increment button;
five `input tap 540 1260` events then screencap showed "Count:
 5".
 Compose recomposition works
end to end on the device.
 Screens saved at `/var/tmp/vet-jc/vetjc-before.png` and `vetjc-after.png`.

Cleanup:
 `adb uninstall com.monochromatic.vetjc` -> Success,
 package removed.

### Bonus: embedded HTTP server inside the Compose app (the heart of this app)

Added Ktor server CIO 3.5.0 (`ktor-server-core`,
 `ktor-server-cio`) started from `onCreate`
via `embeddedServer(CIO, port = 8099, host = "127.0.0.1") { routing { get("/") {...} } }.start(wait = false)`,
with `<uses-permission android:name="android.permission.INTERNET"/>` and core-library
desugaring (`isCoreLibraryDesugaringEnabled = true` + `com.android.tools:desugar_jdk_libs:2.1.5`;
the `l8DexDesugarLibDebug` task ran,
 so desugaring engaged).
 Build succeeded.

First on-device run crashed with:

```text
java.net.SocketException: Operation not permitted   (EPERM)
  at sun.nio.ch.Net.socket0 ... io.ktor.network.sockets ... ktor-server-cio bind
```

Root cause:
 GrapheneOS.
 It exposes the INTERNET permission as a user-revocable "Network"
toggle that is OFF by default for freshly installed apps,
 and returns EPERM (not stock
Android's impossible-to-deny behavior,
 and not EACCES) when denied.
 Granting it:

```bash
adb shell pm grant com.monochromatic.vetjc android.permission.INTERNET
adb shell dumpsys package com.monochromatic.vetjc | grep INTERNET
#   android.permission.INTERNET: granted=true, flags=[USER_SENSITIVE_WHEN_GRANTED|..._DENIED]
```

Then re-run,
 `adb forward tcp:18099 tcp:8099`,
 curl over the forward:

```bash
curl -s -i http://127.0.0.1:18099/
#   HTTP/1.1 200 OK
#   Content-Type: text/plain; charset=UTF-8
#   hello-from-ktor-cio-on-android
curl -s -i http://127.0.0.1:18099/health     # HTTP/1.1 200 OK ... OK
```

So Ktor CIO runs a real HTTP server inside a Compose Android app and serves correct responses.
The EPERM was NOT an NIO/Ktor defect;
 it was the GrapheneOS Network toggle.
 This is a direct
finding for the real app:
 on the owner's GrapheneOS device the app must prompt the user to
enable Network access,
 or the local S3 server cannot bind.

## 3. Ecosystem survey (libraries + versions) for what this app needs

### Embeddable local HTTP server (kopia's S3 target)

- Ktor server CIO 3.5.0 (`io.ktor:ktor-server-core`,
   `io.ktor:ktor-server-cio`).
   Recommended.
  Coroutine-native,
   streaming request/response bodies (needed to proxy S3 PUT/GET to pCloud
  without staging a second copy),
   pure-Kotlin CIO engine (no Netty).
   Verified on-device above.
  Needs core-library desugaring at minSdk 24.
   Apache-2.0.
- NanoHTTPD 2.3.1 (`org.nanohttpd:nanohttpd`).
   Works (classic blocking `java.net.ServerSocket`,
  no desugaring),
   but see maintenance:
   effectively dormant.
   Acceptable only as a trivial
  fallback,
   not the primary server.
   BSD-3-Clause.

### HTTPS client to the pCloud native API

- OkHttp 5.3.2 (`com.squareup.okhttp3:okhttp`).
   Recommended for the proxy core:
   low-level,
  streaming bodies,
   connection pooling,
   well suited to translating/streaming S3 calls to pCloud
  without buffering.
   Apache-2.0.
- Retrofit 3.0.0 (`com.squareup.retrofit2:retrofit`).
   Optional typed layer over OkHttp if a
  declarative pCloud API surface is wanted;
   not needed for raw streaming.
   Apache-2.0.
- Note:
   Ktor also ships a client (`ktor-client-okhttp`) that wraps OkHttp,
   which could unify the
  stack if preferred.

### Background long-running work (kopia backup runs)

- Foreground Service + WorkManager (`androidx.work:work-runtime` 2.11.2).
   Recommended pattern.
- Constraint to design around:
   Android 14+ (API 34) requires a declared `foregroundServiceType`
  (here `dataSync`).
   Android 15+ (API 35) caps cumulative `dataSync` foreground-service runtime
  at roughly 6 hours per day;
   long backups can hit this.
   The owner should plan for user-initiated
  data-transfer jobs / re-arming,
   or chunked runs.
   This is the single biggest platform constraint
  for the app and is independent of the UI toolkit.

### Bundling + exec of the kopia Go binary on modern Android

- Pattern:
   compile kopia per ABI (arm64-v8a is the Pixel 6 target),
   ship each as a `lib*.so`
  inside the APK `jniLibs/<abi>/`,
   set `android:extractNativeLibs="true"`,
   and exec from the
  read-only `applicationInfo.nativeLibraryDir`.
- Why:
   since API 29 Android enforces W^X and blocks `exec()` of files written to the app's data
  dir or read from assets;
   `nativeLibraryDir` is the supported exec-permitted location (the
  Termux/jniLibs approach).
   This is well-established for shipping Go/C binaries on Android.
- Verified here only that the build packages native libs cleanly (`mergeDebugNativeLibs`,
  `stripDebugDebugSymbols` handled `libandroidx.graphics.path.so`).
   The exec-from-nativeLibraryDir
  step itself was NOT bench-executed in this vet (no kopia binary built);
   this is recall of
  Android platform policy,
   not a measured result.
   Recommend the owner spike this one path early,
  it is the highest-risk integration and is UI-toolkit-independent.

## 4. Maintenance signals (via gh / Google Maven)

- Jetpack Compose / AndroidX:
   AOSP-developed,
   mirror `androidx/androidx` last push 2026-06-06.
  Compose BOM cadence is roughly monthly (2026.02.01,
   .
  03.00,
   .
  03.01,
   .
  04.01,
   .
  05.00,
   .
  05.01).
  Issues are tracked on Google IssueTracker,
   not GitHub,
   so GitHub issue counts are not a signal
  here;
   release cadence and Google's first-party backing are.
   Apache-2.0.
   Healthy.
- Ktor 3.5.0 (2026-05-18).
   14.4k stars,
   last push 2026-06-06,
   202 merged PRs since 2026-03-01,
  monthly releases (3.4.1 Mar,
   3.4.2 Mar,
   3.4.3 Apr,
   3.5.0 May).
   JetBrains-maintained.
   Apache-2.0.
  Active.
- OkHttp 5.3.2.
   47k stars,
   last push 2026-06-06,
   64 merged PRs since 2026-03-01.
   Square/Block.
  Apache-2.0.
   Active.
- Retrofit 3.0.0 (2025-05).
   43.9k stars,
   last push 2026-06-01,
   11 merged PRs since 2026-03-01.
  Mature and slower-moving but maintained.
   Apache-2.0.
- WorkManager `androidx.work` 2.11.2:
   part of AndroidX,
   active.
- NanoHTTPD:
   7.2k stars,
   last push 2023-07-25,
   0 merged PRs since 2024-01-01,
   162 open issues,
  last Maven release 2.3.1 (2019).
   Effectively dormant.
   This is why Ktor CIO is the primary
  server recommendation and NanoHTTPD is only a fallback.

## 5. Repo fit (Monochromatic monorepo: TypeScript + Rust, mise + pnpm + cargo)

A Kotlin/JVM/Gradle module is a self-contained island here.
 mise can own it via a task that
shells out to the Gradle wrapper (the way this vet drove Gradle),
 but pnpm and cargo give it
nothing and it gives them nothing.
 There is no shared build graph,
 no shared lint,
 no shared
test harness (`@monochromatic-dev/test-harness-core` does not apply).
 This island cost is real but it
is identical for native Jetpack Compose and for Compose Multiplatform,
 so it does not decide
between them.
 It is inherent to "an Android app in this repo,
" not to the toolkit choice.

## 6. Native Jetpack Compose vs Compose Multiplatform for THIS app

Both render the same androidx.
compose UI API.
 The differences that matter here:

- Native Compose uses Google's first-party libraries directly and tracks the newest Compose BOM
  and the AGP 9 built-in-Kotlin path immediately (verified on AGP 9.2.1).
   CMP republishes the
  same API under `org.jetbrains.compose` and typically lags AndroidX Compose by a version,
   adding
  the Kotlin Multiplatform plugin and the JetBrains Compose Gradle plugin on top.
- This app is Android-only (owner-stated) and every hard part is Android-platform-specific:
  the embedded server,
   foreground service + 6h dataSync cap,
   jniLibs/nativeLibraryDir exec of
  kopia,
   and the pCloud HTTP client.
   None of that is shareable to iOS/desktop/web,
   so CMP's one
  advantage (multiple targets) buys nothing.
- CMP would add abstraction and a lag layer for zero payoff on a single target.

Pick native Jetpack Compose unless a future iOS/desktop/web client becomes a real requirement.
If that flips,
 CMP is the migration path and the UI code (Composables) largely carries over.

### Alternatives named, with rejection reasons (per choosing-technology)

- Compose Multiplatform:
   rejected for an Android-only app.
   Adds KMP + JetBrains Compose plugin
  complexity and a version-lag layer over first-party AndroidX,
   with no second target to justify
  it;
   the app's core is entirely Android-platform-specific.
- Android Views / XML layouts (the legacy UI toolkit):
   rejected.
   Google's tooling,
   docs,
   and new
  APIs have moved to Compose;
   Views is maintenance-mode for new apps and is more boilerplate than
  the small control UI this app needs (start/stop,
   status,
   permission prompts).
- Flutter / React Native (cross-platform non-JVM UI):
   rejected.
   A non-JVM runtime complicates the
  two integrations this app is built around,
   embedding a JVM/Kotlin HTTP server and exec-ing the
  kopia Go binary from nativeLibraryDir;
   it would fight the very work that defines the app.
