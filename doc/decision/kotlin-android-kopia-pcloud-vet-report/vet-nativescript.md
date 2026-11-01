# Vet: NativeScript (TypeScript/JavaScript, V8 on Android) for the kopia-to-pCloud Android app

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-12
Standard:
 choosing-technology FULL-VERIFICATION (clone + source audit + maintenance signals + build + run on
the real device).
 Build ran in a bounded podman container (`--memory=6g`,
 image
`localhost/nativescript-android-builder` derived from the slint-android-builder base:
 Debian 12 + JDK 17 +
Android SDK + Node 22.22.3 + NativeScript CLI 9.0.6).
 Device steps ran against the shared Pixel 6 (oriole)
under `flock /tmp/agent/adb-phone.lock`.
 Verdict and summary are at the bottom.

## What was vetted

NativeScript builds native-UI apps from TypeScript/JavaScript:
 it embeds its own V8 engine in
`libNativeScript.so`,
 runs the app's bundled JS on it,
 and bridges to Android Java/Kotlin APIs by reflection
over generated metadata.
 It renders real `android.view` widgets (not a WebView).
 For this app the question is
whether that stack can (a) run at all on the owner's GrapheneOS Pixel 6,
 and (b) host the load-bearing
non-UI parts:
 a local S3-compatible HTTP server kopia targets,
 streamed through to pCloud without buffering
whole objects.
 kopia and the pCloud API are owner-decided and out of scope.

Pinned versions actually exercised this session (not metadata):

- `nativescript` CLI 9.0.6 (`npm view nativescript version`),
   `@nativescript/core` ~9.0.0,
   `@nativescript/webpack`
  ~5.0.25,
   `@nativescript/android` runtime **9.0.4** (resolved during `ns prepare`,
   the current `latest`).
- Build output:
   AGP-driven Gradle 8.14.3,
   compileSdk/targetSdk **35**,
   minSdk 24,
   webpack 5.107.2 bundle
  (`vendor.mjs` 7.43 MiB),
   universal debug APK 101 MB across arm64-v8a/armeabi-v7a/x86/x86_64.

Clones (shallow,
 depth 1,
 under `/tmp/agent/`):
 `NativeScript/android` (runtime 9.0.4),
`NativeScript/NativeScript` (CLI + `@nativescript/core` + webpack + Static Binding Generator),
`@nativescript-community/https`,
 `NanoHTTPD`,
 plus GrapheneOS `platform_system_sepolicy`.

## Decisive result: it does NOT run on the owner's GrapheneOS Pixel 6

A minimal app (counter UI + a `java.lang.Runnable`/`Thread` running an in-app `ServerSocket` HTTP server +
a `@nativescript/core` HTTP client hitting it) was built and installed on the device.
 It **crashes at V8
isolate initialization,
 before any app JavaScript runs**,
 killing even this hello-world-scale app.

The runtime loads fine,
 then V8 aborts:

```text
nativeloader: Load .../base.apk!/lib/arm64-v8a/libNativeScript.so using class loader ns clns-9 ... ok
TNS.Runtime: NativeScript Runtime Version 9.0.4, commit e140faf5c52213604308623c232429fa50d991b3
auditd  : type=1499 audit(0.0:41314): TSEC_FLAG_DENY_EXECMEM: op denied, uid 10394, pid 28371, ...
E v8    : # Fatal javascript OOM in MemoryChunk allocation failed during deserialization.
F libc  : Fatal signal 5 (SIGTRAP), code 1 (TRAP_BRKPT), ... in tid 28371 (g.vet.nscounter)
```

Native crash stack (logcat `DEBUG`/tombstone),
 all frames in `libNativeScript.so`:

```text
#00 v8::base::OS::Abort()
#02 v8::internal::V8::FatalProcessOutOfMemory(Isolate*, char const*, bool)
#04 v8::internal::MemoryAllocator::AllocateAlignedMemory(..., v8::internal::Executability, ...)
#06 v8::internal::MemoryAllocator::AllocatePage(..., Executability)
#13 v8::internal::Deserializer<Isolate>::ReadObject(SnapshotSpace)
#17 v8::internal::StartupDeserializer::DeserializeIntoIsolate()
#18 v8::internal::Isolate::Init(...)
#23 tns::Runtime::PrepareV8Runtime(...)
#26 Java_com_tns_Runtime_initNativeScript
#58 com.tns.NativeScriptApplication.onCreate
```

`am start` reports `Status: ok`,
 but `ResumedActivity` stays the launcher
(`app.lawnchair.nightly/app.lawnchair.LawnchairLauncher`),
 `pidof org.vet.nscounter` is empty,
 and Zygote
reports `Process 28371 exited due to signal 5 (Trap)`.
 No `NSVET:` probe log line was ever emitted,
 so no app
JS executed.

### Root cause (kernel-confirmed, not inferred)

The proximate cause is the GrapheneOS kernel line at the exact crash timestamp:
`TSEC_FLAG_DENY_EXECMEM: op denied, uid 10394` (`uid 10394`/`pid 28371` is the app).
 GrapheneOS's "Restrict
dynamic code loading (from memory)" exploit protection **denies executable-memory allocation** for the app.
V8 must allocate an executable (`Executability`) memory chunk for its code space while deserializing the
startup snapshot at isolate init;
 the kernel refuses,
 V8 treats the refused allocation as an out-of-memory
condition (it is not a real OOM,
 the Pixel 6 has gigabytes free) and aborts.

This is a **different mechanism from the Slint disqualifier**.
 Slint hit
`RESTRICT_MEMORY_DCL → java.lang.SecurityException` at `InMemoryDexClassLoader` construction (loading a Java
helper's dex from memory).
 NativeScript hits `DENY_EXECMEM` (the executable-memory / JIT side of the same
memory-DCL protection).
 Both are GrapheneOS hardening,
 but NativeScript's break is in V8's own engine
initialization,
 so it fires for any app,
 not only ones that load a helper.

### The obvious mitigation (`--jitless`) does NOT work

V8 flags are app-settable via `nativescript.config.ts` `android.v8Flags` (source:
 `AppConfig.java:13,94-95` →
`Runtime.cpp:591-592` `V8::SetFlagsFromString`).
 `--jitless` is the textbook way to make V8 not allocate
executable memory.
 It was tested:
 the project was rebuilt with `v8Flags: '--jitless --expose_gc'` (confirmed
baked into the runtime `package.json`),
 reinstalled,
 and launched.
 It crashes with the **identical** signature:

```text
auditd  : TSEC_FLAG_DENY_EXECMEM: op denied, uid 10395, pid 28848
E v8    : # Fatal javascript OOM in MemoryChunk allocation failed during deserialization.
#04 MemoryAllocator::AllocateAlignedMemory(..., Executability, ...)  ->  #17 StartupDeserializer::DeserializeIntoIsolate()
```

The reason is NativeScript's own start-up order,
 not a V8 limitation,
 and it is source-verified
(`Runtime.cpp` in the 9.0.4 clone).
 The app's flag string is read into `Constants::V8_STARTUP_FLAGS` at
`Runtime.cpp:231-232`,
 but it is not pushed to V8 until `V8::SetFlagsFromString(...)` at `Runtime.cpp:591`,
which executes *after* the isolate is created by `Isolate::New(create_params)` at `Runtime.cpp:571` (both
calls are in the same `PrepareV8Runtime` body,
 top to bottom:
 `InitializeV8`/`V8::Initialize` at 567,
 then
`Isolate::New` at 571,
 then `SetFlagsFromString` at 591).
 The crash is at isolate creation (snapshot
deserialization,
 frame `StartupDeserializer::DeserializeIntoIsolate` under `Isolate::New`),
 so the `--jitless`
flag is applied too late to affect it;
 both builds enter `Isolate::New` with V8's default JIT-enabled flags,
which is why the crash is byte-identical.
 So as the runtime ships there is **no app-side V8 flag** that boots
it on this device:
 the only levers would be an upstream NativeScript reorder (apply flags before
`Isolate::New`) or a V8 built with `v8_enable_jitless`,
 neither available to an app developer.
 Because the app
never starts there is no jitless runtime performance to measure here;
 and even a hypothetical reordered jitless
runtime would still owe the separate,
 unproven question of whether interpreter-only V8 is fast enough for an
app this size (its bundle is a 7.43 MiB vendor chunk),
 on top of the unresolved storage-DCL break.

### The only escape, and why it is worse than Slint's

The single escape is the owner disabling the per-app GrapheneOS "Restrict dynamic code loading (from memory)"
toggle for the app,
 re-allowing execmem (verified to exist:
 grapheneos.
org/features lists the three per-app
DCL toggles;
 that disabling it would rescue V8 is inference,
 matching how the Slint vet treated its own
per-app DCL toggle;
 the owner's device security setting was not mutated for this vet).
 Even then,
 a **second,
independent** GrapheneOS break remains.

The NativeScript runtime generates a `.dex` at runtime and loads it via a file-based `DexClassLoader` for any
native subclass the build-time Static Binding Generator (SBG) could not statically resolve (source:
`DexFactory.java:176` and `:396`,
 `ClassResolver.java:21-26` gated on `com.tns.gen*`;
 confirmed by upstream
issue `NativeScript/android#1962`,
 closed 2026-06-05).
 The SBG only pre-generates bindings for native
subclasses written as literal dotted names it can fold statically (`es5-visitors.js:35-46,355-373`);
 the
ubiquitous idioms it cannot resolve (an aliased `const R = java.lang.Runnable; new R({...})`,
 computed class
refs,
 plugins that build natives at runtime,
 a JS-implemented `android.app.Service`) fall through to the
runtime `DexClassLoader`,
 which is exactly what GrapheneOS's **storage**-DCL protection blocks
(`RESTRICT_STORAGE_DCL`,
 active on this device per the Slint vet).
 That path was not reached on-device for
NativeScript because the execmem break (above) kills the process first;
 it is source-verified plus
device-verified-as-active,
 not directly observed for NativeScript.

Net:
 Slint has one break (memory-DCL,
 helper dex) with one toggle as an escape hatch.
 NativeScript has two
stacked breaks (memory-DCL for V8's execmem,
 plus storage-DCL for the runtime `DexClassLoader`),
 needs two
toggles disabled for a realistic app,
 and its only app-side lever (`--jitless`) does not work.

Caveat,
 labeled inference:
 on stock Android,
 or on a GrapheneOS install with the memory + storage DCL toggles
off for this app,
 the same APK would very likely boot and render.
 The runtime loaded,
 V8 reached
deserialization,
 and the sole failure is the kernel `DENY_EXECMEM` denial,
 which is a GrapheneOS feature absent
on stock Android.
 A stock-device / emulator run was not performed,
 so that is inference,
 not verified;
 but the
kernel-denial line makes the mechanism GrapheneOS-specific,
 not a build defect.

## Build: what worked (full verification of the build path)

The build path itself is clean and current,
 which makes the run failure the decisive evidence:

- Scaffold:
   `ns create nscounter --js --appid org.vet.nscounter` → standard Core project
  (`App_Resources/Android/src/main/AndroidManifest.xml` already declares INTERNET;
   jniLibs would live at
  `App_Resources/Android/src/main/jniLibs/`).
- Build:
   `ns build android --apk` → `Platform android successfully added. v9.0.4`,
   webpack compiled in 1.3 s,
  Gradle build 49.6 s,
   `app-debug.apk` produced.
   Runtime AAR is `nativescript-optimized-with-inspector` (debug).
- **16 KB page alignment is correct out of the box.
  ** `readelf -l lib/arm64-v8a/libNativeScript.so` shows all
  LOAD segments aligned `0x4000` (16 KB),
   so the runtime is already Android 15/16-compatible with no manual
  fix.
   This is better than the Slint/cargo-apk path,
   which needed a hand-injected
  `-Wl,-z,max-page-size=16384`.
   Source corroboration:
   the runtime builds with
  `-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON` (`runtime/build.gradle:117`);
   STL is `c++_static`,
   so no separate
  `libc++_shared.so` is bundled.
- Platform currency:
   the 9.0.4 runtime targets compileSdk/targetSdk **35 (Android 15)**,
   not 36;
   it runs on
  the Pixel 6's Android 16 in compatibility mode.
   The runtime is actively maintained (stable 9.0.4 published
  2026-04-29,
   `9.0.5-next` builds dated 2026-06-12,
   `9.1.0-alpha`).

So nothing about the toolchain or the APK is stale or broken;
 the stack is simply incompatible with the
device's hardening.

## The gateway has no natural home in NativeScript (source-audited)

Even setting the boot failure aside,
 the app's load-bearing component is a poor fit.
 The streaming pump moves
bytes from the inbound kopia socket straight to the outbound pCloud HTTPS body with no whole-object buffering;
the decisive question is which language runs that per-chunk copy loop.

- `@nativescript/core` `Http` is a buffering CLIENT,
   not a server.
   It exposes only
  `request/getString/getJSON/getFile/...` (`packages/core/http/index.ts`),
   the transport is
  `java.net.HttpURLConnection` (not OkHttp),
   the request body is written whole
  (`Async.java:398 outStream.write(((ByteBuffer) content).array())`),
   and the response is read fully into a
  `ByteArrayOutputStream2` with an in-source TODO admitting "this approach will not work for very large files"
  (`Async.java:477-478`).
   Unusable for the gateway in either direction.
- To host a server you add a Maven/AAR dep in `App_Resources/Android/.../include.gradle` and reference Java
  classes from JS (proven in-tree:
   `@nativescript-community/https` pulls `okhttp3` and drives it from JS).
   The
  viable servers are `java.net.ServerSocket` (hand-roll HTTP/1.1 + the S3 verbs),
   NanoHTTPD (unmaintained
  since 2019,
   and its convenience body API buffers/temp-files),
   or Ktor CIO (current,
   but a Kotlin coroutine
  server).
   In every case the streaming copy loop must run in **Java/Kotlin**,
   because the NativeScript JS↔Java
  bridge copies byte buffers O(n) on every crossing (`http-request-internal/index.android.ts:158-160`
  `ByteBuffer.wrap(Array.from(typedArray))`),
   so a pump authored in JS is disqualifying.
- The one JS-resident option,
   `nodejs-mobile` (Node's `http` + streams),
   means embedding a **second** JS engine
  (`libnode.so` ~62 MB per ABI) beside V8;
   its NativeScript binding `nodejs-mobile-nativescript` is gone from
  npm and GitHub,
   and the core fork's last release is 2024-10-07.

Bottom line:
 every workable gateway path puts the byte pump in Kotlin/Java and uses NativeScript purely as a
launcher,
 i.e. you write the same Kotlin (Ktor + OkHttp) a Jetpack build would write,
 inside a JS shell that
adds nothing to the load-bearing path.
 Tauri/Slint give the gateway a first-class Rust home (axum + reqwest);
Jetpack gives it a first-class Kotlin home (Ktor + OkHttp).
 NativeScript's strength (native UI from TS) lands
on the app's least important part while the hard part has no home.

## Maintenance and testing (source-audited)

- Core stack:
   active releases,
   thin support.
   `NativeScript/NativeScript` 25.5k stars,
   MIT,
   pushed 2026-06-12;
  `@nativescript/core` 9.0.20 (2026-05-27),
   monthly cadence;
   CLI 9.0.6.
   But 837 open issues,
   ~37% of recent
  issues get no maintainer reply,
   merge throughput is concentrated in a handful of nStudio-aligned authors with
  a multi-year stale external-PR pile.
   Governance:
   Telerik → Progress → nStudio stewardship (2020) → OpenJS,
  now re-tiered down to **At-Large** (OpenJS's lowest tier).
   Funding is an OpenCollective at ~$16.7k/yr,
   i.e.
  effectively subsidized by nStudio's commercial work.
   The runtime tracks roughly one Android major behind
  (targets API 35;
   Android 16/API 36 is user-driven and partly broken:
   `#10851` open,
   `#11001` "targetSdk 36:
  app is getting closed").
- Testing:
   unit tests run on device via `@nativescript/unit-test-runner` (Karma + Jasmine/Mocha/QUnit,
   no
  Jest;
   the runner was dormant ~3.5 years then bumped for v9).
   The element-location plumbing for black-box E2E
  is genuinely in core (`automationText`/`testID` → Android `setTag`/content-desc:
  `view-common.ts:978-983`,
   `index.android.ts:1409-1419`),
   so Maestro/Appium can drive it,
   but the maintained
  framework integration is missing:
   the official `@nativescript/detox` plugin is a husk (3 releases ever,
   does
  not even pin `detox`),
   Appium tooling is years-stale,
   and Maestro is a vendor-blog "build-your-own-harness"
  story.
   Contrast:
   the Kotlin stacks get `createAndroidComposeRule` plus first-class Maestro/Appium.

## Comparison and ranking

For the Android-only kopia-pCloud app on the owner's GrapheneOS Pixel 6,
 in this TS + Rust monorepo:

Native Jetpack Compose > Tauri v2 > Compose Multiplatform > Slint > **NativeScript** (last).

- vs Slint (the stack immediately above):
   both fail to run on the device,
   but Slint keeps the gateway in
  first-class Rust (matching the repo's existing desktop Slint apps),
   has a single break (memory-DCL) with one
  toggle as an escape hatch,
   and is the repo's incumbent UI tech.
   NativeScript has two stacked GrapheneOS
  breaks (execmem/JIT plus runtime-dex/storage-DCL),
   its only app-side mitigation (`--jitless`) does not work,
  and the gateway has no Rust or idiomatic-Kotlin home (it forces Kotlin/Java plus a JS shell that adds
  nothing).
   NativeScript also embeds a second JIT engine (V8),
   which is the very thing GrapheneOS blocks.
- The one place NativeScript could win,
   a TypeScript-friendly UI suiting this TS-heavy repo (the same edge
  Tauri's web UI has),
   is undercut twice:
   the hard part (gateway) cannot be TS,
   and the app does not boot.

Flip condition (stated honestly):
 for a different target device with no DCL hardening (stock Android,
 or a
GrapheneOS install with the memory + storage DCL toggles off for this app),
 NativeScript would boot and run,
and its TS-friendly UI would put it in roughly Tauri's class (JS/TS UI plus a native shell,
 gateway pushed to
Kotlin).
 But the target is the owner's hardened GrapheneOS Pixel 6,
 where it is disqualified,
 more decisively
than Slint.

## Friction log (every obstacle hit, in order)

1. No Node / `ns` CLI / newer SDK platform in the base image:
    derived `localhost/nativescript-android-builder`
   from the slint base,
    adding Node 22 + `nativescript@9.0.6` + platforms android-34/35 (one-time setup cost).
2. SELinux on the Fedora host denied the unlabeled bind mount (`EACCES` on `ns create`);
    fixed with `:Z` on
   the `-v` volume.
3. Each ephemeral `--rm` build container generated its own random debug keystore
   (`/root/.android/debug.keystore` not persisted),
    so the second APK failed `INSTALL_FAILED_UPDATE_INCOMPATIBLE`
   (signature mismatch);
    resolved by `adb uninstall` before reinstalling.
    Persisting `/root/.android` as a
   volume would avoid it for repeat builds.
4. The decisive obstacle:
    V8 cannot initialize on the device (`TSEC_FLAG_DENY_EXECMEM`),
    and `--jitless` does
   not fix it.
    No app-side workaround exists.

What worked:
 the toolchain installed cleanly;
 `ns create`/`ns build` produced a valid,
 16 KB-aligned,
current-runtime APK on the first try;
 the failure is entirely the device's executable-memory hardening,
 not
the build.

## Verdict

NativeScript for the Android kopia-pCloud app on the owner's GrapheneOS Pixel 6:
 **DISQUALIFIED,
 ranked last
of the five stacks.
**

- Toast on the target device:
   built a current,
   16 KB-aligned NativeScript 9.0.4 APK in a bounded container and
  ran it on the physical Pixel 6 (Android 16/API 36).
   It crashes at V8 isolate init
  (`TSEC_FLAG_DENY_EXECMEM` → "Fatal javascript OOM ... during deserialization" → SIGTRAP),
   before any app JS,
  killing even a hello-world-scale app.
   The obvious mitigation `--jitless` was built and run and crashes
  identically;
   no app-side V8 flag rescues it.
- Two stacked GrapheneOS breaks,
   vs Slint's one:
   memory-DCL/execmem (device-verified) and
  storage-DCL/runtime-`DexClassLoader` (source-verified,
   active on this device).
   The only escape is the owner
  disabling per-app exploit-protection toggles (two of them for a realistic app),
   weakening that app's
  hardening;
   the device security settings were not mutated for this vet.
- Even discounting the boot failure,
   the gateway (the app's hard part) has no natural home in NativeScript:
  `@nativescript/core` HTTP is a buffering client with no server and no streaming,
   and the JS↔Java bridge
  copies buffers O(n) per crossing,
   so the streaming pump must be Kotlin/Java regardless,
   with NativeScript as
  a do-nothing launcher.
- Maintenance is active-but-thin (OpenJS At-Large,
   nStudio-subsidized,
   ~one Android major behind),
   and the
  E2E testing story is build-your-own-harness.

Artifacts (under `/var/tmp/nativescript-vet-work/`):
 project `nscounter/`;
 APK
`nscounter/platforms/android/app/build/outputs/apk/debug/app-debug.apk`;
 `logcat-run1.txt` (default JIT build
crash),
 `logcat-jitless.txt` (jitless build crash,
 identical signature).
 Image
`localhost/nativescript-android-builder`.

## Summary (<400 words)

Android build+run result:
 build PASS,
 run FAIL on the physical Pixel 6 (oriole,
 Android 16/API 36,
 GrapheneOS,
patch 2026-06-06).
 `ns create nscounter --js` then `ns build android --apk` produced a universal debug APK
(101 MB,
 runtime `@nativescript/android` 9.0.4,
 compile/targetSdk 35),
 with `libNativeScript.so` already
16 KB-page-aligned (`readelf` LOAD `0x4000`),
 no manual fix.
 On the device,
 under flock-guarded adb:
`adb install` Success,
 `adb shell pm grant ... INTERNET`,
 `am start` Status ok,
 but the runtime loads
(`TNS.Runtime 9.0.4`) and then V8 aborts at isolate init:
`auditd: TSEC_FLAG_DENY_EXECMEM: op denied, uid 10394` →
`E v8: Fatal javascript OOM in MemoryChunk allocation failed during deserialization` →
`Fatal signal 5 (SIGTRAP)`,
 stack `OS::Abort ← FatalProcessOutOfMemory ← MemoryAllocator::AllocateAlignedMemory(..,Executability,..) ← StartupDeserializer::DeserializeIntoIsolate ← tns::Runtime::PrepareV8Runtime ← NativeScriptApplication.onCreate`.
Process SIGKILL'd,
 no UI,
 no app JS (no probe log line).
 Uninstalled cleanly.

Root cause (kernel-confirmed):
 GrapheneOS "Restrict dynamic code loading (from memory)" denies executable
memory;
 V8 needs an executable chunk for its code space while deserializing the startup snapshot,
 the kernel
refuses,
 V8 aborts as OOM.
 This is the executable-memory/JIT mechanism,
 distinct from Slint's
`InMemoryDexClassLoader` `SecurityException`.
 The `--jitless` mitigation was built and run and crashes
identically because NativeScript applies app `v8Flags` via `SetFlagsFromString` (`Runtime.cpp:591`) after it
creates the isolate with `Isolate::New` (`Runtime.cpp:571`),
 so the flag never affects the crashing isolate
init;
 no app-side flag boots it,
 and there is nothing to benchmark because it never starts.
 A second,
 independent break remains:
 NativeScript generates dex
at runtime via `DexFactory → DexClassLoader` (`DexFactory.java:176,396`,
 issue android#1962) for native
subclasses the build-time SBG cannot statically resolve,
 which GrapheneOS storage-DCL (active on this device)
blocks.
 So NativeScript has two stacked breaks vs Slint's one,
 and its only app-side mitigation fails.

Architecture fit:
 the gateway has no natural home.
 `@nativescript/core` HTTP is a buffering client (no server,
no streaming;
 `Async.java:398`);
 the JS↔Java bridge copies buffers O(n) per crossing
(`index.android.ts:158-160`),
 so the streaming pump must be Kotlin/Java (ServerSocket/Ktor + OkHttp) regardless
of NativeScript.
 Maintenance:
 active-but-thin (OpenJS At-Large,
 nStudio-subsidized,
 ~one Android major behind,
targets API 35).
 Testing:
 on-device Karma unit tests;
 E2E is a build-your-own-Maestro harness.

Verdict:
 DISQUALIFIED for the target device,
 ranked last of five (below Slint):
 it does not boot on the owner's
GrapheneOS Pixel 6,
 `--jitless` does not save it,
 it has two GrapheneOS breaks,
 and the gateway would be Kotlin
anyway.
