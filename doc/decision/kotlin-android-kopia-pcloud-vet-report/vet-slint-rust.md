# Vet: Slint + Rust as the stack for an Android-only app (kopia + local S3 to pCloud)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Scope:
 assess the STACK (Slint UI + Rust ecosystem) for an Android-only app.
 kopia and the
pCloud API are out of scope;
 the binary-bundling and on-device-server mechanics are assessed
because they are stack concerns.
 The repo already ships Slint + Rust for two desktop apps
(package/terminal/app,
 package/music-player/desktop-app),
 pinned to a slint git rev;
this vet covers the NEW part:
 Android.

Standard applied:
 choosing-technology FULL-VERIFICATION (clone + source audit + maintenance
signals + build + run on a real device).
 Verification ran in a bounded podman container
(--memory=8g --cpus=4) on /var/tmp;
 the device install ran on the host under
flock /tmp/agent/adb-phone.
lock against the shared Pixel 6 (oriole).

## 1. Slint Android backend: source audit

Clone:
 `gh repo clone slint-ui/slint /tmp/agent/slint-vet-20260607 -- --depth 1` (slint 1.17.0,
master HEAD;
 crates.
io newest published is 1.16.1).

There is a dedicated,
 first-class Android backend,
 not a winit afterthought:

- `internal/backends/android-activity/` is a complete `i_slint_core::platform::Platform`
  implementation (`lib.rs`):
   event loop driven by `AndroidApp::poll_events`,
   timers/animations,
  `EventLoopProxy` (`invoke_from_event_loop`/`quit`),
   clipboard,
   color scheme (light/dark),
  accent color,
   long-press interval.
- `Cargo.toml`:
   gated on `cfg(target_os = "android")`;
   renders with **Skia only**
  (`i-slint-renderer-skia`),
   uses `android-activity` 0.5 and 0.6 (features `aa-05`/`aa-06`),
  `jni` 0.22,
   `ndk` 0.8/0.9,
   `raw-window-handle` 0.6.
   FemtoVG is explicitly disabled on Android
  (api/rs/slint/`Cargo.toml` comment:
   needs RUST_FONTCONFIG_DLOPEN).
   So Android => Skia => host
  `clang` needed at build time.
- `javahelper.rs` (710 lines) + `java/SlintAndroidJavaHelper.java`:
   real IME integration
  (`InputConnection`,
   preedit/composition for CJK),
   soft-keyboard show/hide,
   text-selection
  cursor handles,
   copy/paste/cut `ActionMode` menu,
   window insets,
   night-mode callback.
   This is
  mature touch/text UI work,
   well beyond "it draws a rectangle".
- Public API `api/rs/slint/android.rs`:
   `slint::android::init(app)` /
  `init_with_event_listener`,
   `android_main` entry,
   re-exports `android-activity`.
   Min SDK 26
  (doc/astro guide).
   cargo-apk default NativeActivity manifest matches the feature
  (`backend-android-activity-06` => native-activity).

Tests / CI:
 `.github/workflows/ci.yaml` job `android` builds 5 crates for
`aarch64-linux-android` via cargo-apk on every relevant change (todo,
 energy-monitor,
printerdemo,
 usecases,
 slint-viewer).
 FINDING:
 CI is **compile-only** — no emulator / no
on-device instrumented run.
 That makes a real-device run (this vet) the decisive evidence.
CHANGELOG shows sustained Android fixes across many releases (initial aa-05 support → aa-06 →
IME,
 selection handles,
 safe-area,
 wgpu,
 Back key,
 most recent "IME keyboard not appearing"
#11357).
 No Android-specific fuzz/property harness (the renderer is Skia,
 fuzzed upstream).

## 2. Rust ecosystem fit (crates + versions, crates.io as of 2026-06-07)

- Local S3 HTTP server:
   **axum 0.8.9** on **hyper 1.10.1** over **tokio 1.52.3**.
   Pure-Rust,
  no platform deps;
   binds 127.0.0.1:
  PORT.
   tokio is a supported Android target.
   Fit:
   strong.
- HTTPS client with STREAMING to pCloud:
   **reqwest 0.13.4** with `rustls-tls` (avoids
  cross-compiling OpenSSL for the NDK).
   Streaming up via `Body::wrap_stream`,
   down via
  `.bytes_stream()`.
   Direct precedent:
   Slint's own `energy-monitor` example depends on
  `reqwest 0.12` (rustls-tls) + `tokio "full"` **with `backend-android-activity-06`** — i.e.
  reqwest+tokio already compile and run under Slint on Android in-tree.
   Fit:
   strong.
- Background execution:
   tokio runs in the activity process;
   a long backup that survives
  backgrounding needs an Android **foreground service** (FGS,
   type dataSync on API 34+).
   That is
  a manifest/Kotlin concern (see build-tool finding below),
   language-neutral.
- Bundling + exec of the kopia Go binary:
   language-neutral Android platform constraint.
   On
  API 29+ you may NOT exec a file from the app data dir (W^X);
   the executable must live in the
  native-lib dir,
   be extracted (`android:extractNativeLibs="true"`),
   and be exec'd from
  `getApplicationInfo().nativeLibraryDir` (set `LD_LIBRARY_PATH` for non-system deps).
   The Pixel
  6 is API 33+,
   so this applies.
   From Rust:
   `jni` 0.22 to read `nativeLibraryDir`,
   then
  `std::process::Command`.
   cargo-apk supports this partly via `runtime_libs` (bundles extra
  `.so` into the APK lib dir) and the `extractNativeLibs` manifest flag.
   ndk 0.9.0,
   jni 0.22.4.

## 3. Maintenance signals (gh + crates.io, 2026-06-07)

- slint-ui/slint:
   22,831 stars,
   pushed today,
   not archived.
   PRs merged same-day by core
  maintainers (ogoffart,
   tronical = the founders);
   issues triaged within days;
   Android backend
  touched 2026-06-06.
   crates.
  io 1.16.1 (2026-04-23) lags master 1.17 — hence the repo's git-rev
  pin.
   Health:
   excellent.
- HTTP/async crates:
   tokio 32k★ (2026-05),
   hyper 16k★/1.10.1 (2026-05),
   axum 26k★/0.8.9
  (2026-04),
   reqwest 11.6k★/0.13.4 (2026-05).
   All current,
   massive download counts.
   Excellent.
- android-activity (rust-mobile):
   387★,
   0.6.1 published 2026-03-24 ("Release 0.6.1",
   "Update to
  thiserror 2"),
   19M downloads.
   Low-churn but actively maintained;
   the shared base for
  winit/bevy/egui on Android.
   Healthy.
- jni 0.22.4 (2026-03),
   ndk 0.9.0 (2024,
   stable/low-churn).
   Healthy.
- **cargo-apk (rust-mobile):
   the weak link.
  ** 201★;
   crates.
  io newest **0.10.0 published
  2023-11-30** (>2 years stale);
   recent merged PRs in-repo are dependabot-only (2023);
   several
  2026 issues (e.g. #74 "doesn't support Android Platform 36.1",
   #84 UID error) sit without
  maintainer comments.
   `cargo install cargo-apk` installs the 2023 binary — which still builds
  (Slint CI and this vet both use it) but is frozen.
- xbuild (the documented alternative):
   665★ but pushed 2025-08;
   Slint's own docs warn 0.2.0 is
  "severely outdated,
   use the git version".
   More stale than cargo-apk.

Interpretation:
 the UI framework and the server/HTTPS/async crates are all robustly maintained.
Risk is concentrated entirely in the Rust→APK packaging tool,
 and it is mitigable (see verdict).

## 4. cargo-apk capability ceiling (source: rust-mobile/cargo-apk @ /tmp/agent/cargo-apk-20260607)

`ndk-build/src/manifest.rs` models the manifest as one `Application` containing exactly one
`Activity`.
 It exposes `extract_native_libs`,
 `uses_cleartext_traffic`,
 `uses_permission`,
`runtime_libs`.
 There is **no `<service>` element**.
 Consequence:
 the pure cargo-apk path can
ship a single-Activity Slint app and bundle the kopia binary,
 but cannot declare the foreground
service a long-running backup needs.
 A production build should therefore use the **Gradle +
cargo-ndk** path:
 cargo-ndk builds the Rust `.so`,
 the Android Gradle project owns the full
manifest (FGS + permissions),
 places kopia under `jniLibs/arm64-v8a/`,
 and loads Slint's
`android_main`.
 Slint documents the Gradle route for C++ already;
 it is the same packaging for a
Rust `.so`.
 cargo-apk stays fine for prototyping.

## 5. FULL VERIFICATION (build + run on Pixel 6)

Minimal app written:
 `/var/tmp/slint-vet-work/counter-app` (cdylib,
 `android_main`,
 a Slint
window with a title,
 a "Count:
 N" label,
 and an Increment button wired to a Rust callback that
increments `counter`).
 Toolchain (bounded podman container,
 image `localhost/slint-android-builder`):
Debian rust:
bookworm + JDK 17 + clang + Android SDK platform-33/build-tools-33.0.2 + NDK
26.3.11579264 + `cargo install cargo-apk` (=> cargo-apk 0.10.0,
 the 2023 release) +
`rustup target add aarch64-linux-android`.
 Container run:
`podman run --rm --memory=8g --cpus=4 -v counter-app:/work -v cargo-registry:/usr/local/cargo/registry -w /work localhost/slint-android-builder cargo apk build --target aarch64-linux-android --lib`.

BUILD — three iterations,
 each a finding:

1. `slint = { version = "1.16", default-features = false, features = ["compat-1-2",
   "backend-android-activity-06"] }` => **compile error in slint 1.16.1 itself**:
   `slint-1.16.1/android.rs:157: cannot find type 'Box' in this scope` (E0433).
    The released
   crate's Android entry omits the no_std `Box` import;
    master 1.17 fixes it by fully-qualifying
   `alloc::boxed::Box`.
    This is exactly why the repo pins a slint git rev rather than crates.
   io.
2. Re-enabling default features (=> `std`) compiled cleanly.
    Output:
   `target/debug/apk/slintcounter.apk` (~150 MB debug;
    signed with the auto debug keystore;
   contains `lib/arm64-v8a/libslintcounter.so`).
3. First device run surfaced an **Android 16 "16 KB page size" compatibility warning**:
   `libslintcounter.so : LOAD segment not aligned` (cargo-apk 0.10.0 + NDK r26 emit 4 KB-aligned
   LOAD segments;
    Android 15+/16 flags them).
    Fixed by injecting
   `RUSTFLAGS=-Clink-arg=-Wl,-z,max-page-size=16384` (cargo-apk appends to existing RUSTFLAGS,
   verified in ndk-build/src/`cargo.rs`).
    `readelf -l` then showed all LOAD aligns = `0x4000`
   (16 KB).
    Rebuild produced a clean,
    16 KB-aligned APK.

RUN — on the shared Pixel 6 (oriole),
 all adb steps under `flock /tmp/agent/adb-phone.lock`.
Device:
 `Pixel 6`,
 Android **16 / API 36**,
 1080x2400,
 arm64-v8a,
 security patch 2026-06-01,
fingerprint `google/oriole/oriole:16/BP4A.251205.006/2026060101`.
 The device runs a hardened ROM
(GrapheneOS:
 Vanadium browser,
 microdroid payload,
 Lawnchair;
 and `DynCodeLoading: AppBindFlags:
RESTRICT_MEMORY_DCL, RESTRICT_STORAGE_DCL, RESTRICT_WEBVIEW_DCL`).

- `adb install -r slintcounter.apk` => Success.
   `.so` loads:
   `nativeloader: Load
  .../libslintcounter.so ... ok`.
- `adb shell am start -W -n dev.vet.slintcounter/android.app.NativeActivity` => Status ok,
   but the
  activity never becomes visible/resumed;
   `topResumedActivity` stays the launcher.
   Process is
  spawned then **killed (signal 9)**.
- **Crash,
   captured from logcat** (RustStdoutStderr):
  `thread panicked at i-slint-backend-android-activity-1.16.1/androidwindowadapter.rs:186:68:
  JNI error: CaughtJavaException { ... name: "java.lang.SecurityException" }`,
   immediately
  preceded by `DynCodeLoadingUtils: handleAppReportedDcl, denialType: InMemoryDexFile, pkg:
  dev.vet.slintcounter`.
   Notification:
   "Slint Counter Vet tried to perform DCL via memory".
- Cleanup:
   `adb uninstall dev.vet.slintcounter` => Success,
   `pm path` empty (removed).

ROOT CAUSE (source-pinned):
 `internal/backends/android-activity/javahelper.rs:251-307`
(`get_helper_class_loader`) embeds `SlintAndroidJavaHelper` as a build-time `classes.dex`
(`DEX_DATA = include_bytes!(.../classes.dex)`) and ALWAYS loads it via **dynamic code loading** —
`InMemoryDexClassLoader` on API ≥ 26,
 file-based `DexClassLoader` below.
 There is no path that
resolves the helper from the app's own classpath.
 The window adapter (`androidwindowadapter.rs:186`)
calls `JavaHelper::new(...).unwrap_or_else(print_jni_error)` during window construction,
 so the
SecurityException from GrapheneOS's DCL restriction is fatal and the app never renders.

CORROBORATION:
 GrapheneOS (build 2024083100+) ships "Restrict dynamic code loading" (memory and
storage) ON by default;
 unauthorized DCL raises `SecurityException` at `InMemoryDexClassLoader.<init>`;
a per-app toggle exists (Settings → Apps → app → Exploit protection) with a security trade-off.
Many mainstream apps are affected.
 No matching Slint issue exists upstream (searched
"dynamic code loading",
 "GrapheneOS",
 "SecurityException dex" => 0 results),
 so there is no fix in
flight and the user would be the first reporter.

VERDICT ON THE RUN:
 the BUILD works (Rust → 16 KB-aligned APK via cargo-apk).
 The RUN **fails hard
on the user's Pixel 6** (GrapheneOS) — a reproducible startup crash with no UI.
 Per the brief's
rule ("if the Android build/run is impractical,
 that is a DISQUALIFYING finding for the Android
target"),
 this disqualifies stock-Slint for this device as-shipped.
 Caveat,
 labeled as inference:
the crash is solely at the DCL gate (the `.so` loaded and Skia/Vulkan began initializing),
 so on
stock Android / OEM ROMs / the AOSP emulator (where DCL is allowed,
 as in Slint's compile-only CI)
the same APK would very likely render;
 I did not run it on a stock device,
 so that is inference,
not verified.
 Mitigations:
 (a) user flips the per-app GrapheneOS DCL toggle off (manual,
 per-install,
weakens that app's hardening);
 (b) an upstream Slint change to load the helper from a classes.
dex
on the app classpath (needs the Gradle path AND a Slint patch) — not available today.

## 6. Alternatives compared

Two rankings,
 because they disagree:

- By developer ergonomics / repo-fit (cargo + mise,
   one language,
   no second runtime):
  **Slint > Tauri v2 > Compose.
  **
- By "does it actually run on the user's GrapheneOS Pixel 6 today" (§5):
   **Tauri v2 ≈ Compose >
  Slint.
  ** Slint fails on the device due to its dynamic-code-loading helper;
   Compose (pure native
  Kotlin in `classes.dex`,
   no DCL) and Tauri (Kotlin glue in `classes.dex`;
   system WebView) do
  not trigger the DCL restriction.
   The run reality is decisive,
   so the practical recommendation
  flips away from Slint for this device unless the DCL issue is resolved.

### Slint + Rust (the candidate)

- Pros:
   whole app stays in Rust (UI + axum server + reqwest streaming + kopia exec);
   one
  compiled binary,
   no second runtime,
   no JS/npm;
   native repo fit (cargo + mise,
   no Gradle/JVM);
  first-class Android backend with real IME/touch maturity;
   Skia rendering is consistent across
  devices (not WebView-version-dependent).
- Cons:
   Skia => host clang + heavier first build;
   APK packaging tool (cargo-apk) is stale and
  single-Activity-only,
   so a production build with a foreground service needs the Gradle +
  cargo-ndk path;
   released crates.
  io build (1.16.1) had an Android compile bug (see §5),
   so a
  git-rev pin is effectively required (which the repo already does for desktop).

### Tauri v2 (web UI + Rust) — second

- Pros:
   keeps the Rust core (server/streaming/exec identical to Slint);
   Android stable since 2.0
  (2024),
   mobile is first-class in 2026;
   UI in HTML/CSS/JS suits a TS-heavy team;
   its
  `gen/android` Gradle project gives full manifest control (foreground service,
   permissions) out
  of the box,
   which cargo-apk lacks.
- Cons:
   UI runs in Android System WebView — a second runtime whose behavior varies by device
  WebView version;
   adds a JS bundler island (Vite/npm/node_modules) on top of cargo,
   i.e. a
  second build system the monorepo would otherwise avoid.
- Why Slint ranks above Tauri:
   both keep the Rust core,
   but Slint ships one compiled binary with
  no WebView and no web toolchain;
   Tauri's sole real edge (web UI familiarity) is minor for a
  small settings/progress UI,
   and it costs a second runtime + bundler.

### Compose Multiplatform (Kotlin/JVM) — third

- Pros:
   best-in-class Android UI (Jetpack Compose underneath:
   Material 3,
   accessibility,
   system
  integration),
   turnkey foreground service + WorkManager scheduling,
   trivial `jniLibs` binary
  bundling,
   full manifest control,
   largest talent pool and best tooling (Android Studio);
   very
  actively maintained by JetBrains.
- Cons:
   a Kotlin/JVM + Gradle island — exactly the foreign build system the repo wants to avoid.
  The app's hard part is Rust-shaped (local server,
   streaming,
   exec),
   so you must either rewrite
  it in Kotlin (Ktor + OkHttp + ProcessBuilder,
   abandoning Rust and the monorepo's shared code)
  or keep it in Rust and maintain a JNI bridge across two languages and two build systems.
  Heavier JVM/ART + Compose runtime.
- Why Tauri ranks above Compose:
   Tauri keeps the app's hard part in one language (Rust);
   Compose
  forces a JVM/Gradle island plus a rewrite-or-JNI-bridge for the very functionality that is the
  app's reason to exist.
- Flip condition (stated honestly):
   if the app instead demanded deep native-feeling Material UI,
  accessibility,
   or rich system integration,
   Compose's native Android UI maturity would outrank
  both — and its Gradle project makes the foreground-service + binary-bundling story the most
  turnkey of the three.
   For a Rust-core background utility,
   that advantage does not dominate.

## Summary

Android build+run (exact path):
 built a minimal Slint+Rust counter app (cdylib,
 `android_main`,
button + label) in a bounded podman container (cargo-apk 0.10.0,
 NDK r26,
 slint 1.16.1) via
`cargo apk build --target aarch64-linux-android --lib`.
 Three build findings:
 (1) released slint
1.16.1 won't compile its own `android.rs:157` under `default-features=false` (missing no_std
`Box`;
 fixed on master) — enable `std`;
 (2) cargo-apk + NDK r26 emit 4 KB-aligned LOAD segments
Android 16 flags as not 16 KB-compatible — fixed with
`RUSTFLAGS=-Clink-arg=-Wl,-z,max-page-size=16384` (readelf confirmed `0x4000`);
 (3) signed 16 KB
APK produced.
 On the Pixel 6 (Android 16/API 36,
 GrapheneOS-hardened),
 under flock-guarded adb:
install Success,
 `.so` loads,
 but `am start` crashes at startup —
`panic at i-slint-backend-android-activity/androidwindowadapter.rs:186: JNI error ...
java.lang.SecurityException` after `handleAppReportedDcl ... InMemoryDexFile`,
 process SIGKILL'd,
no UI.
 Uninstalled cleanly.

Slint Android maturity:
 first-class backend (dedicated platform impl,
 Skia rendering,
real IME/CJK composition,
 selection handles,
 dark mode,
 accent color,
 min SDK 26),
 built in
Slint's CI for 5 crates via cargo-apk,
 sustained fixes across many releases,
 excellent upstream
responsiveness.
 BUT its Java helper is loaded only via dynamic code loading
(`InMemoryDexClassLoader`/file `DexClassLoader`;
 `javahelper.rs`:
251-307),
 with no app-classpath
fallback — which GrapheneOS's default exploit protection blocks.
 This is the disqualifying run
failure for the target device;
 it is unreported upstream (no fix in flight).

Rust ecosystem fit:
 strong and DCL-free.
 Local S3 server:
 axum 0.8.9 / hyper 1.10.1 / tokio
1.52.3 (pure Rust,
 binds 127.0.0.1).
 HTTPS streaming to pCloud:
 reqwest 0.13.4 + rustls-tls;
reqwest+tokio already ship under Slint-on-Android in the energy-monitor example.
 Background needs
a foreground service,
 which cargo-apk's single-Activity model cannot declare — use Gradle +
cargo-ndk.
 Binary exec:
 API 29+ allows exec only from `nativeLibraryDir`;
 cargo-apk `runtime_libs`
bundles the `.so`,
 exec via a jni-read path + `std::process`.
 All current,
 well-maintained.

Repo-fit:
 best of the three on paper (cargo + mise,
 one language,
 matches the desktop Slint apps),
but the UI does not run on the user's device.

Verdict:
 Slint+Rust is a clean stack for the server/HTTP/exec core,
 but **stock-Slint's UI does
not run on the user's GrapheneOS Pixel 6** (reproducible SecurityException from its
dynamic-code-loaded Java helper).
 For this device,
 recommend **Tauri v2** (keeps the Rust core,
runs on GrapheneOS) over Compose (best Android UI but a full JVM/Gradle island),
 and adopt Slint
only if (a) the device's per-app DCL restriction is disabled,
 or (b) Slint upstream gains an
app-classpath helper path.
 Do not commit to Slint for Android until the DCL crash is resolved.
