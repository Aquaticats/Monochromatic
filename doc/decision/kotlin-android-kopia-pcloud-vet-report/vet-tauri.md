# Vet: Tauri v2 (mobile/Android) plus Rust ecosystem for the kopia-to-pCloud Android app

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07
Standard:
 choosing-technology skill,
 FULL-VERIFICATION (clone plus source-audit plus build plus run on the real device).
Verdict and a sub-400-word summary are at the bottom.

## What was vetted

A web-frontend (TS/HTML/CSS) plus Rust-backend stack for an Android-only app that runs kopia on-device,
exposes a local S3-compatible HTTP server kopia targets,
 and streams S3 to pCloud's native HTTP API without
duplicating storage on the phone.
 kopia and the pCloud API are owner-decided and out of scope;
 this assesses
the STACK only.

Pinned versions actually exercised this session (not metadata):

- tauri 2.11.2,
   tauri-cli 2.11.2,
   tauri-build 2.6.2 (`crates/tauri/Cargo.toml:version`,
   `crates/tauri-cli/Cargo.toml:version`).
- Webview/window:
   wry 0.55.1,
   tao 0.35.3 (Cargo.
  lock;
   built in the APK run below).
- Generated Android toolchain:
   AGP 8.11.0,
   Kotlin Gradle plugin 1.9.25,
   Gradle 8.14.3,
   compileSdk/targetSdk 36,
  NDK 29.0.13846066,
   default minSdk 24 / Android 7
  (`crates/tauri-cli/templates/mobile/android/{build.gradle.kts,app/build.gradle.kts,gradle/wrapper/gradle-wrapper.properties}`,
  `crates/tauri-cli/src/mobile/android/mod.rs:51` NDK_VERSION,
   `crates/tauri-utils/src/config.rs:3254` default minSdk 24).
- Android plumbing crate:
   cargo-mobile2 0.22.4 (Cargo.
  lock).

Clones (shallow,
 depth 1):

- `/tmp/agent/tauri-vet` -> tauri-apps/tauri (HEAD `66f873d6` 2026-06-06 "fix(mobile):
   avoid mutex deadlocks #15491").
- `/tmp/agent/plugins-workspace-vet` -> tauri-apps/plugins-workspace.

## 1. Ecosystem audit

### Android support is part of the stable v2 line (not a separate track)

README badge is `status-stable`;
 the platform table lists Android "7 and above (currently 8 and above)" and iOS
"9 and above" alongside desktop (`README.md:3,51-52`).
 The same v2 crates serve desktop and mobile;
 there is no
"mobile beta" fork.
 The Android runtime is a real,
 substantial Kotlin codebase under
`crates/tauri/mobile/android/src/main/java/app/tauri/` (PluginManager.
kt,
 plugin/Invoke.
kt,
 plugin/Channel.
kt,
JniMethod.
kt,
 PermissionHelper.
kt,
 AppPlugin.
kt,
 PathPlugin.
kt).
 The webview/IPC bridge lives in wry at
`wry-0.55.1/src/android/kotlin/` (Ipc.
kt,
 Rust.
kt,
 RustWebView*.
kt,
 WryActivity.
kt).
 So Android is first-class,
not a thin wrapper.

### How Rust commands are exposed to the TS frontend (verified on device, see 3)

`#[tauri::command]` (`crates/tauri-macros/src/lib.rs:35`) plus `tauri::generate_handler![...]` register Rust fns;
the JS side calls `window.__TAURI__.core.invoke("name", args)` (when `app.withGlobalTauri=true`) or the
`@tauri-apps/api` package.
 On Android the call crosses JS -> Android System WebView -> Kotlin IPC bridge
(wry `Ipc.kt`) -> JNI -> Rust,
 with `tauri-runtime-wry` wiring `with_ipc_handler` and a custom protocol
(`crates/tauri-runtime-wry/src/lib.rs:5163`).
 Channels (`plugin/Channel.kt`) provide event streaming back to JS.
`#[cfg_attr(mobile, tauri::mobile_entry_point)]` (`crates/tauri-macros/src/lib.rs:40`) is the mobile entry hook.

### Rust async HTTP server (local S3 endpoint) and streaming HTTPS client (pCloud): cross-compiled for Android

This is the load-bearing technical question and it passes with direct evidence.
 A probe crate
(`/var/tmp/tauri-vet-work/probe`) mirroring the gateway shape (an axum router with a streaming PUT handler plus a
reqwest+rustls client that streams a body via `reqwest::Body::wrap_stream`) cross-compiled cleanly for
`aarch64-linux-android`:

```text
cargo build --target aarch64-linux-android   # NDK aarch64-linux-android24-clang as linker
   Compiling tokio v1.52.3 ... tokio-rustls v0.26.4 ... h2 v0.4.14 ... hyper v1.10.1
   Compiling hyper-rustls v0.27.9 ... axum v0.8.9 ... reqwest v0.12.28
    Finished `dev` profile in 25.98s   (libandroidprobe.so produced)
```

Crate stack proven for arm64-v8a:
 axum 0.8.9,
 hyper 1.10.1,
 h2 0.4.14 (HTTP/2),
 tokio 1.52.3 (full),
tower-http 0.6.11,
 reqwest 0.12.28.
 TLS resolves to pure-Rust rustls 0.23.40 + ring 0.17.14 + hyper-rustls
0.27.9 with NO openssl-sys and NO native-tls in the tree (`cargo tree` filtered).
 That matters:
 reqwest's
default native-tls path needs a cross-compiled OpenSSL,
 which is the classic Android pain;
 rustls avoids it
entirely and `ring` builds for Android out of the box.
 The streaming bodies (`body.into_data_stream()` server
side,
 `Body::wrap_stream` client side) are exactly the "do not duplicate storage" path:
 stream an S3 PUT
straight into a pCloud upload with backpressure.
 Separately,
 the Tauri app build itself already compiles
hyper/tokio/tower/reqwest 0.13.4 for `aarch64-linux-android` as part of its own dependency tree (build log
lines for tokio 1.52.3,
 hyper 1.10.1,
 tower-http 0.6.11,
 reqwest 0.13.4).

### Bundling and exec of the kopia Go binary: the real constraint

Tauri's first-class sidecar mechanism (`externalBin`) is DESKTOP-ONLY.
 In the source,
 externalBin is bundled
only by `tauri-bundler` for Windows/macOS (`crates/tauri-bundler/src/bundle/{windows,macos}/app.rs`);
 the Android
build path has no externalBin handling at all.
 The Android resource injector
(`crates/tauri-cli/src/mobile/android/mod.rs:953 inject_resources`) copies only `bundle.resources` into the APK
`assets/` directory (read-only,
 served via AssetManager,
 not an executable filesystem path).
 The official shell
plugin's `sidecar()` exists (`plugins/shell/src/lib.rs:67`) and its `execute`/`spawn` use
`std::process::Command` (`plugins/shell/src/commands.rs:206`),
 but nothing places an external executable into the
APK for Android,
 and modern Android (API 29+) refuses to `execve` files from the app's writable data dir
(W^X / SELinux).
 The supported path is the well-known Android trick,
 done by hand in the generated
`gen/android` project (which Tauri hands you to edit):
 ship the kopia arm64 binary as `libkopia.so` under
`app/src/main/jniLibs/arm64-v8a/`,
 keep `extractNativeLibs=true`,
 then exec it from
`applicationInfo.nativeLibraryDir/libkopia.so` via `std::process::Command` in Rust.
 This is viable and standard
(it is how Termux-style apps ship binaries),
 but it is NOT a Tauri feature you get for free;
 budget for the
jniLibs packaging plus a manifest tweak.
 This identical "exec a bundled binary on Android" constraint applies to
ANY Android stack (Kotlin/Compose included);
 it is not a Tauri-specific penalty.

### Android background-execution constraints under Tauri

Tauri gives you ONE foreground `MainActivity` (singleTask) hosting a WebView plus a FileProvider,
 and nothing
else:
 the template manifest declares no `<service>`,
 no `FOREGROUND_SERVICE`,
 no wakelock
(`crates/tauri-cli/templates/mobile/android/app/src/main/AndroidManifest.xml`).
 To keep the local S3 server plus
kopia running while the app is backgrounded or the screen is off,
 you must add a Kotlin foreground Service
(plus `FOREGROUND_SERVICE`/`FOREGROUND_SERVICE_DATA_SYNC` permissions and a persistent notification) to the
generated project yourself;
 Tauri provides no abstraction for it.
 Maintainers confirm this is the app's job:
on issue #15485 ("Android broadcasts") member FabianLars points the reporter at Android's background-work docs
rather than a Tauri API.
 Again,
 this is an Android-platform reality,
 not Tauri-unique,
 but Tauri does nothing to
soften it,
 whereas a Kotlin app sits natively in that world.

## 2. Maintenance signals (via gh)

Tauri core (tauri-apps/tauri):
 107,589 stars,
 3,676 forks,
 Apache-2.0,
 not archived,
 pushed 2026-06-07 (today),
~1,448 open issues (large but active).
 Release cadence is tight and synchronized:
 the full crate set published
2026-05-16 (tauri/tauri-cli 2.11.2,
 tauri-runtime-wry 2.11.2,
 tauri-bundler 2.9.2,
 etc.).
 Recent merged PRs are
core-maintainer driven with live mobile work:
 #15496 @lucasfernog (lead),
 #15491 "fix(mobile):
 avoid mutex
deadlocks" 2026-06-06,
 #15444 "perf(mobile):
 reuse dev reqwest client" @Legend-Master 2026-05-29,
 plus
@amrbashir,
 @FabianLars.
 Issue responsiveness is real:
 members/contributors reply within hours-to-days
(FabianLars on #15492/#15485/#15478,
 Legend-Master on #15483),
 with triage cross-linking (#15472).

Android backlog shape (open,
 label "platform:
 Android"):
 #14994 "Android IPC block until next IPC arrive",
#15337 "Android 15 crashes with security reinforcement technique",
 #15419 "android run builds both APK and AAB",
#15385 java/gradle version hint,
 #15153 Dioxus hot-reload.
 So Android has genuine bugs,
 but they are being
actively fixed:
 the repo HEAD commit is literally the mobile-deadlock fix (#15491) that addresses the IPC-stall
class.
 Interpretation:
 actively maintained with a healthy,
 triaged backlog;
 Android IPC reliability has needed
ongoing fixes (a watch item,
 not a disqualifier).

plugins-workspace (official plugins):
 1,747 stars,
 Apache-2.0,
 pushed 2026-06-07,
 ~446 open issues,
 regular
maintainer merges and a bot-driven "Publish New Versions (v2)" release flow (#3425).
 Healthy.

License:
 Apache-2.0 / MIT across tauri and plugins-workspace.
 Clean for repo use.

Source-audit notes (code-quality signals):
 tauri has an in-repo `audits/` and `supply-chain/` (cargo-vet)
directory and a large `crates/tauri/tests` plus `examples/` set;
 CI is public GitHub Actions.
 The Android
runtime is plain Kotlin with annotation-driven plugin registration.
 No Rust-level fuzz harness was surfaced in
the spot-read of the mobile crates (not unusual for a framework runtime;
 report absence inline).

## 3. Full verification (build plus run on a physical Pixel 6)

Toolchain installed under an isolated working dir `/var/tmp/tauri-vet-work` (no host pollution):
 Temurin JDK 17
(via mise),
 Android cmdline-tools 11076708,
 platform-tools,
 platforms;
android-36,
 build-tools;
36.0.0,
NDK 29.0.13846066,
 cargo-tauri 2.11.2 (binstalled),
 rustup targets aarch64/armv7/i686/x86_64-linux-android
(nightly 1.98 active toolchain).
 Device:
 Pixel 6 (oriole),
 arm64-v8a,
 Android 16 / API 36.
 All device steps were
`flock /tmp/agent/adb-phone.lock`-guarded (shared phone).
 All builds on /var/tmp disk.

### 3a. Scaffold

Static frontend `app/frontend/index.html` (button + counter,
 no build step),
 then:

```text
cargo-tauri init --ci --app-name taurivet --window-title "Tauri Vet" --frontend-dist ../frontend
# edited src-tauri/tauri.conf.json: withGlobalTauri=true, identifier com.taurivet.app, removed npm hooks
# edited src-tauri/src/lib.rs: added #[tauri::command] greet + stateful increment(State<Counter=Mutex<i64>>)
cargo-tauri android init      # victory: Project generated successfully!
```

`android init` auto-installed the remaining Rust android targets and placed the build under
`src-tauri/gen/android` (Gradle project,
 editable).

### 3b. Build the APK (debug, aarch64): PASS

```text
cargo-tauri android build --debug --apk --target aarch64
   Finished `dev` profile [unoptimized + debuginfo] in 40.53s        # Rust, incl. hyper/tokio/reqwest 0.13.4
   Info symlinking lib .../libapp_lib.so in jniLibs dir .../arm64-v8a # Rust lib -> jniLibs (the same trick kopia needs)
   Downloading gradle-8.14.3-bin.zip ... 100%
   Finished 1 APK at:
     .../gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

APK = 135 MB debug (unstripped debug `.so`;
 a `--release` strip would be vastly smaller).
 Wall clock ~3.5 min
(20:51:23 -> 20:54:50) including the one-time Gradle download.
 Only non-fatal deprecation warnings (Gradle
`exec`,
 WebView `databaseEnabled`,
 `onBackPressed`).

### 3c. Install, launch, confirm resumed, verify the JS<->Rust bridge, screencap: PASS

```text
adb -s 1C171FDF600KWW install -r app-universal-debug.apk          -> Success
adb shell pm list packages | grep taurivet                        -> package:com.taurivet.app.debug
adb shell monkey -p com.taurivet.app.debug -c android.intent.category.LAUNCHER 1  -> Events injected: 1
adb shell dumpsys activity activities | grep ResumedActivity
  -> ResumedActivity: ...com.taurivet.app.debug/com.taurivet.app.MainActivity   (RESUMED)
adb shell pidof com.taurivet.app.debug                            -> 21233 (running)
```

On-device functional proof (uiautomator dump of the WebView),
 initial state:

```text
text="0"
text="Increment (Rust state)"
text="Hello Android, from Rust on Android!"   # return value of invoke("greet",{name:"Android"}) -> Rust ran
```

That single line is the IPC proof:
 JS called a Rust command,
 Rust executed,
 the string came back and rendered.
Then 3 taps on the button (bounds [86,718][992,952],
 center 539,835):

```text
adb shell input tap 539 835   x3
# re-dump:
text="3"
text="rust counter = 3"        # stateful increment() Mutex<i64> went 0->1->2->3, each value returned to JS
```

Screens captured and pulled:
 `/var/tmp/tauri-vet-work/taurivet1.png` (counter 0,
 greet line) and
`taurivet2.png` (counter 3,
 "rust counter = 3").
 Content rendered correctly below the status bar (a plain
WebView body,
 so none of the edge-to-edge clipping the Compose vet hit).
 Cleanup:

```text
adb uninstall com.taurivet.app.debug   -> Success
adb shell rm -f /sdcard/uidump*.xml /sdcard/taurivet*.png
pm list packages | grep taurivet       -> (none; package removed)
```

So the full integration boundary is crossed on real hardware:
 TS webview UI + bidirectional,
 stateful Rust
command bridge,
 building from `cargo tauri android build` to a running,
 RESUMED activity on a Pixel 6.

## 4. Comparison vs Compose Multiplatform and Slint+Rust

Context:
 the repo has a same-day decision doc (`doc/decision/kotlin-android-kopia-pcloud-stack.md`) that already
accepted a Kotlin stack (Compose MP + Ktor + OkHttp + Kotest + Jazzer) for THIS app,
 explicitly calling it
"a Gradle/JVM/Android island inside a mise plus pnpm plus Rust monorepo.
" Tauri is the alternative that removes
that island.
 Both Compose MP and a Tauri app were independently built and run on the same Pixel 6 this session.

### vs Compose Multiplatform (Kotlin/JVM): `/tmp/agent/vet-compose.md`

- Repo fit:
   Compose MP is a JVM/Gradle island;
   it shares none of the repo's TS/Rust/oxlint/tsgo/dprint/tsdown
  toolchain (the decision doc says so).
   Tauri's frontend is the repo's native TS/HTML/CSS and its backend is the
  repo's native Rust;
   the local-S3-server + pCloud-client logic would be the SAME Rust crates the monorepo
  already builds (axum/reqwest/tokio,
   proven above),
   reusable as a normal workspace crate.
   This is Tauri's
  decisive advantage for THIS repo.
- Android maturity:
   roughly even,
   both verified on the device.
   Compose MP IS Google's Jetpack Compose (AOSP
  fork) so its Android rendering is best-in-class;
   Tauri renders in the Android System WebView.
   For a
  backend-heavy "gateway with a thin UI" app,
   WebView is more than enough and the UI is throwaway-cheap in HTML.
- Adoption friction:
   Compose MP cost the other vet a real AGP-9 migration (broken official template,
   must move to
  `com.android.kotlin.multiplatform.library`,
   requires JDK 21,
   tight Kotlin/compose-compiler/AGP/Gradle version
  coupling,
   edge-to-edge inset fixes).
   Tauri's `cargo tauri android init/build` worked first try with no manual
  Gradle surgery (only an editable generated project and benign deprecation warnings).
- The gateway half:
   Compose MP's UI does not address the server;
   the decision doc pairs it with Ktor (CIO server
  + OkHttp engine).
     Tauri pairs naturally with axum + reqwest-rustls in the same Rust process as the UI backend.
- Rejection reason for THIS app:
   it forces a second language runtime and a separate build island for an
  Android-only utility whose hard part is Rust/HTTP,
   not UI.
   Compose MP earns its keep when a rich native UI must
  span phone+desktop+iOS;
   that is not this app's need.

### vs Slint + Rust (Slint is the repo's desktop UI incumbent, pinned in `mise.toml`)

- Repo fit:
   excellent on the backend (all-Rust,
   single language with the gateway crates),
   and Slint is already a
  repo dependency.
   But the UI layer is a `.slint` markup DSL,
   NOT the repo's TS/HTML/CSS,
   so it does not reuse the
  repo's web-frontend strength the way Tauri does.
- Android maturity:
   this is the disqualifier.
   Slint's Android support is comparatively immature (the Compose vet
  reached the same conclusion),
   it relies on its own winit/Skia-style Android Activity glue,
   and it lacks Tauri's
  battle-tested System-WebView IPC bridge and plugin ecosystem.
   I did not get an independent Slint-on-Pixel-6 run
  this session,
   so I am not claiming parity-of-evidence;
   on maturity-of-Android-target alone Tauri is ahead.
- Where Slint wins:
   a pure-Rust,
   no-webview,
   small-footprint binary with no JS at all.
   If the UI were trivial and
  you wanted zero web stack,
   Slint+Rust is the leaner shape.
   For an app that wants to lean on the repo's TS/CSS
  frontend skills and a mature mobile shell,
   Tauri is the better fit.

Ranking for THIS Android-only kopia-pCloud app,
 in this TS+Rust repo:
Tauri v2 > Compose Multiplatform > Slint+Rust.
- Tauri > Compose MP:
   identical Android-run evidence,
   but Tauri keeps everything in the repo's native TS+Rust and
  reuses the gateway Rust crates,
   with far less build friction;
   Compose MP adds a JVM/Gradle island and a second
  language for an app whose hard part is not the UI.
- Compose MP > Slint+Rust:
   Compose MP has materially more mature Android support and was verified on the device;
  Slint's Android target is the weakest of the three and its UI DSL reuses neither the repo's TS nor (for the UI)
  its Rust idioms.

Other alternatives noted for completeness (rejected,
 per the Compose vet's survey):
 Flutter (adds Dart,
 a third
runtime),
 React Native (weak Linux-desktop story,
 but desktop is moot here;
 still a JS-bridge perf variability vs
Tauri's in-process Rust),
 native Jetpack Compose (Android-only,
 same JVM-island cost without the multiplatform
upside).

## 5. Friction log (every obstacle hit, in order)

1.  cargo-binstall placed `cargo-tauri` without the exec bit after a cross-device copy fallback ("Permission
    denied");
     `chmod +x` fixed it.
     Trivial,
     environment-specific.
2.  No JDK/Android SDK/NDK/cargo-tauri preinstalled;
     had to install JDK 17 + cmdline-tools + platform-36 +
    build-tools-36 + NDK-29 + cargo-tauri.
     Standard mobile setup cost,
     not Tauri-specific.
3.  The generated `tauri.conf.json` ships `beforeDevCommand: npm run dev` / `beforeBuildCommand: npm run build`;
    for a no-bundler static frontend these had to be removed or `android build` would invoke a missing npm script.
4.  `withGlobalTauri` is off by default;
     had to enable it to use `window.__TAURI__.core.invoke` without the
    `@tauri-apps/api` npm package.
     (Real apps using a bundler would instead import the api package.
    )
5.  The default app template no longer ships a sample command;
     the counter/greet commands and `invoke_handler`
    were added by hand (expected).
6.  Binary-exec on Android:
     no Tauri sidecar path for mobile (section 1) -> the kopia binary needs the manual
    jniLibs/`libkopia.so` packaging.
     The single largest design task this stack leaves to you.
7.  Background execution:
     no Tauri Service abstraction -> a Kotlin foreground Service must be hand-added.
8.  Android IPC reliability has a live bug class (#14994,
     fixed-at-HEAD #15491) -> pin to a release that includes
    the deadlock fix and watch mobile IPC issues.
9.  Debug APK is large (135 MB) due to unstripped Rust debug symbols;
     use `--release` (and per-abi splits) for any
    real artifact.

What worked well:
 `cargo tauri android init`/`build` worked first try with zero manual Gradle migration (contrast
the Compose AGP-9 ordeal);
 the JS<->Rust command bridge worked on the real device immediately;
 the entire
gateway crate stack (axum/reqwest-rustls/tokio/hyper/h2) cross-compiles for arm64 with pure-Rust TLS and no
OpenSSL;
 the generated Android project is yours to edit for the jniLibs/Service work.

## Verdict

Tauri v2 for the Android kopia-pCloud app:
 VIABLE,
 recommended for THIS repo,
 with two scoped caveats.

- Not toast:
   scaffolded,
   built a debug APK,
   and ran it on a physical Pixel 6 (Android 16) with a working,
  stateful JS<->Rust command bridge (counter 0->3,
   greet round-trip),
   uninstalled cleanly.
   Build path had far
  less friction than the Compose Multiplatform vet on the same device.
- Ecosystem fit is strong where it matters:
   the local S3 server (axum/hyper) and the streaming pCloud client
  (reqwest + rustls,
   no OpenSSL) cross-compile for aarch64-linux-android and live in the same Rust process as the
  UI backend;
   the streaming-body shape matches the no-duplicate-storage constraint.
- Repo fit is the headline:
   Tauri keeps the whole app in the monorepo's native TS+CSS (frontend) and Rust
  (backend,
   reusing workspace crates),
   removing the JVM/Gradle island the accepted Kotlin stack introduces.
- Two caveats to budget,
   both Android-platform realities Tauri does not abstract:
   (a) the kopia Go binary must be
  shipped via the jniLibs `libkopia.so` trick and exec'd from `nativeLibraryDir` (Tauri's sidecar is
  desktop-only);
   (b) keeping the server/kopia alive in the background needs a hand-written Kotlin foreground
  Service.
   Also pin a tauri release including the mobile IPC deadlock fix (#15491).

Artifacts:
 app under `/var/tmp/tauri-vet-work/app`;
 cross-compile probe under `/var/tmp/tauri-vet-work/probe`;
APK at `app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`;
screenshots `/var/tmp/tauri-vet-work/taurivet{1,2}.png`;
 build log captured this session.

## Summary (<400 words)

Android build+run result:
 PASS on a physical Pixel 6 (oriole,
 Android 16/API 36).
 Commands and output:
`cargo-tauri init --ci ...` then `cargo-tauri android init` (-> "Project generated successfully") then
`cargo-tauri android build --debug --apk --target aarch64` (-> "Finished ... in 40.53s",
 "1 APK at
.../app-universal-debug.
apk",
 135 MB debug,
 ~3.5 min total).
 Device:
`adb install -r` -> Success;
 `monkey ... LAUNCHER 1` -> Events injected:
 1;
`dumpsys activity activities | grep ResumedActivity` ->
`...com.taurivet.app.debug/com.taurivet.app.MainActivity` (RESUMED),
 `pidof` -> 21233.
 The WebView rendered
"Hello Android,
 from Rust on Android!
" (return of `invoke("greet")` -> Rust executed),
 and 3 taps drove a
stateful Rust `Mutex<i64>` counter to "rust counter = 3".
 Uninstalled cleanly afterward.

Tauri v2 Android maturity:
 Android is part of the STABLE v2 line (README status-stable;
 Android 7+/minSdk 24),
with a real Kotlin runtime (`app.tauri.*` PluginManager/Invoke/Channel) and a wry System-WebView IPC bridge.
Maintenance is healthy:
 107k stars,
 Apache-2.0,
 pushed today,
 synchronized 2.11.2 releases,
 lead maintainers
actively merging mobile fixes (HEAD = #15491 "fix(mobile):
 avoid mutex deadlocks");
 Android IPC reliability is a
watch item (#14994),
 being fixed.

Ecosystem fit:
 local-server + HTTPS-streaming PASS:
a probe of axum 0.8.9 + reqwest 0.12.28 (rustls 0.23.40 +
ring,
 NO OpenSSL) + hyper 1.10.1 + h2 + tokio 1.52.3 cross-compiled for aarch64-linux-android in 26s;
 streaming
bodies fit the no-duplicate-storage constraint.
 Binary-bundling CAVEAT:
Tauri's sidecar is desktop-only;
 the
kopia binary must be hand-shipped as `jniLibs/arm64-v8a/libkopia.so` and exec'd from `nativeLibraryDir`
(`std::process::Command`),
 an Android-wide constraint,
 not Tauri-unique.
 Background CAVEAT:
Tauri provides only
a foreground Activity;
 a Kotlin foreground Service must be hand-added to keep the server/kopia alive backgrounded.

Repo fit:
 best of the three.
 Tauri keeps the app in the monorepo's native TS+CSS frontend and Rust backend
(reusing workspace gateway crates),
 avoiding the JVM/Gradle island the accepted Kotlin stack introduces.
Ranking for this Android-only app in this TS+Rust repo:
 Tauri v2 > Compose Multiplatform > Slint+Rust (Tauri:
same device-proof as Compose with native repo-language fit and less build friction;
 Compose:
 mature Android but a
JVM island;
 Slint:
 all-Rust but immature Android and a non-web UI DSL).

Verdict:
 VIABLE,
 recommended for this repo.
 Budget the jniLibs binary-exec packaging,
 a Kotlin foreground
Service,
 and pinning a release with the mobile IPC deadlock fix.
