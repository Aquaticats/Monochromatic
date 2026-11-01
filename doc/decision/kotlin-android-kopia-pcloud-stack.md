# Android UI/app-shell stack vet for the kopia-to-pCloud backup app

Status:
 per-technology vet and five-stack comparison complete.
 UI/app-shell stack decision is open
(not yet recorded).
 Date updated:
 2026-06-12.

## Context

The planned app is Android-only.
 It runs kopia on the phone and backs up to pCloud without staging a
second copy on the device.
 kopia speaks S3,
 so the app stands up a local S3 endpoint that kopia targets
and translates those calls to pCloud's native API,
 streaming bytes through rather than buffering whole
objects.

Owner-decided and out of scope of this vet:
 running kopia on Android,
 the pCloud native API,
 and the
necessity of a custom S3 gateway.

Standard:
 choosing-technology full-verification.
 Every candidate was cloned,
 source-audited,
 and actually
built and run,
 in a bounded container or on the real device,
 not judged from metadata.
 The target device
is the owner's Pixel 6 (oriole),
 GrapheneOS,
 Android 16 / API 36.
 The repo is a TypeScript plus Rust
monorepo (mise,
 pnpm,
 cargo).

## Decisive result: run on the real device

All five UI stacks were built and launched on the physical Pixel 6:

- Native Jetpack Compose:
   runs.
   Counter increments;
   an in-app Ktor CIO server served HTTP 200.
- Compose Multiplatform:
   runs (desktop runComposeUiTest plus a debug APK on the device).
- Tauri v2:
   runs.
   The JavaScript to Rust command bridge works on device.
- Slint plus Rust:
   builds,
   but the UI crashes at startup.
   Slint's Android backend loads its Java helper
  only via dynamic code loading (`internal/backends/android-activity/javahelper.rs:251-307`,
  `InMemoryDexClassLoader`),
   which GrapheneOS blocks by default,
   raising a `SecurityException` before any
  UI renders.
   Disqualified for this device as shipped;
   the issue is unreported upstream.
- NativeScript:
   builds (a current,
   16 KB-aligned `@nativescript/android` 9.0.4 APK),
   but crashes at V8
  isolate initialization before any app JavaScript runs.
   GrapheneOS denies executable memory
  (`TSEC_FLAG_DENY_EXECMEM` in logcat),
   so V8 cannot allocate its code space while deserializing the startup
  snapshot and aborts (`Fatal javascript OOM ... during deserialization`,
   SIGTRAP).
   This is the
  executable-memory side of the same protection,
   distinct from Slint's dex-loading `SecurityException`.
   The
  obvious mitigation,
   `--jitless` via `android.v8Flags`,
   was built and run and crashes identically,
   so no
  app-side flag boots it.
   A second,
   independent break also applies (the runtime generates dex at runtime via
  `DexFactory` to `DexClassLoader`,
   which storage-DCL blocks).
   Disqualified for this device as shipped,
  ranked last of the five;
   the failure is unreported upstream.

## Candidate stacks

### Native Jetpack Compose (Kotlin)

Runs on the device.
 All-Kotlin:
 Compose UI,
 Ktor CIO as the local S3 server (verified in-app on the
device),
 OkHttp 5.3.2 / Retrofit 3.0.0 as the pCloud client (Retrofit pulls stable OkHttp 4.12.0),
WorkManager plus a dataSync Foreground Service for the backup.

Pros:
 one language and one build system,
 no FFI bridge;
 the app's hard parts (foreground service,
permissions,
 kopia binary exec,
 GrapheneOS handling) are first-class in Kotlin;
 strongest verified testing
(createAndroidComposeRule in-process plus Maestro black-box).
 Cons:
 a Kotlin/JVM/Gradle island in a
TypeScript plus Rust monorepo;
 the S3 gateway is written in Kotlin rather than reusing Rust.

### Tauri v2 (Rust core plus web UI)

Runs on the device.
 The S3 gateway stays in Rust (axum plus reqwest,
 verified streaming),
 the UI is
HTML/CSS/TypeScript,
 and a `gen/android` Gradle project provides the Android manifest and services.

Pros:
 keeps the gateway in Rust matching the monorepo;
 web UI suits the repo's TypeScript strength.
Cons:
 most polyglot of the options (Rust plus TypeScript plus Kotlin glue;
 cargo plus a JS bundler plus
Gradle);
 the UI runs in the Android System WebView;
 the desktop UI-test driver (tauri-driver) is
self-labeled pre-alpha,
 Linux and Windows only,
 with no native click support and no Playwright attach
path on WebKitGTK.
 It still needs the Gradle/Kotlin shell for the foreground service.

### Compose Multiplatform (Kotlin, multi-target)

Runs on the device.
 Same `androidx.compose` UI as native Jetpack plus the multiplatform layer and
runComposeUiTest.
 For an Android-only app the multiplatform layer adds version coupling and an AGP 9
migration off the stale official template for no benefit over native Jetpack.
 It earns its place only if
iOS or desktop becomes a real target.

### Slint plus Rust

Best repo fit on paper (pure Rust,
 matching the existing Slint desktop apps),
 and the Rust ecosystem core
is strong and DCL-free.
 But the Slint UI does not run on the owner's GrapheneOS device (see above),
 which
disqualifies it for the target device unless the per-app dynamic-code-loading restriction is disabled or
Slint upstream gains an app-classpath helper path.

### NativeScript (TypeScript/JavaScript, V8 on Android)

Runs the app's bundled TypeScript on an embedded V8 in `libNativeScript.so` and renders native Android views
through a reflection bridge.
 TypeScript-friendly,
 which suits the repo on paper.

Does not run on the owner's GrapheneOS device,
 more decisively than Slint.
 V8 needs executable memory to
deserialize its startup snapshot at isolate init;
 GrapheneOS's memory-DCL protection denies it
(`TSEC_FLAG_DENY_EXECMEM`),
 and V8 aborts before any app JS,
 so even a hello-world app dies.
 The textbook
`--jitless` mitigation was built and run and crashes identically because NativeScript applies app `v8Flags`
after it creates the isolate (`SetFlagsFromString` at `Runtime.cpp:591` runs after `Isolate::New` at `:571`),
so the flag never affects the crashing isolate init;
 no app-side flag boots it,
 and there is no jitless
performance to weigh because nothing starts.
 A second,
 independent GrapheneOS break stacks on top:
 the
runtime generates dex at runtime via `DexFactory` to a file-based `DexClassLoader` for any native subclass
the build-time Static Binding Generator
cannot statically resolve,
 which storage-DCL blocks.
 So NativeScript faces two breaks where Slint faces one.
Separately,
 the gateway (the app's hard part) has no natural home:
 `@nativescript/core` HTTP is a buffering
client with no server and no streaming,
 and the JS to Java bridge copies buffers per crossing,
 so the
streaming pump must be Kotlin/Java (Ktor plus OkHttp) regardless,
 with NativeScript as a do-nothing launcher.

Pros:
 native UI from TypeScript suits the repo's strength;
 the build path is clean and the shipped
`libNativeScript.so` is already 16 KB-page-aligned for Android 15/16 with no manual fix.
 Cons:
 disqualified on
the target device (V8 cannot get executable memory,
 and `--jitless` does not help);
 a second storage-DCL break
for realistic native subclassing;
 the gateway lands in Kotlin/Java anyway under a JS shell that adds nothing;
it embeds a second JIT engine,
 the very thing GrapheneOS blocks;
 maintenance is active-but-thin (OpenJS
At-Large tier,
 nStudio-subsidized,
 roughly one Android major behind,
 targets API 35),
 and the E2E testing
story is build-your-own-Maestro-harness.

## Ranking (analysis, not a recorded decision)

Native Jetpack Compose > Tauri v2 > Compose Multiplatform > Slint > NativeScript.

- Native Jetpack over Tauri:
   every viable stack needs a Gradle/Kotlin Android shell anyway for the
  foreground service and kopia bundling,
   so Tauri does not escape the JVM/Gradle island;
   it adds Rust plus
  a JS bundler on top of it.
   Native Jetpack is the only single-language,
   single-build option,
   with the
  best Android integration and the strongest verified testing.
   Its cost is rewriting the gateway in Kotlin,
  which was verified to work on the device.
- Tauri over Compose Multiplatform:
   Tauri is a genuinely different architecture (Rust core,
   web UI);
  Compose Multiplatform is a heavier native Jetpack for an Android-only app.
- Compose Multiplatform over Slint:
   Slint does not run on the device.
- Slint over NativeScript:
   neither runs on the device,
   but Slint keeps the gateway in first-class Rust
  (matching the repo's desktop Slint apps),
   has a single GrapheneOS break with one toggle as an escape hatch,
  and is the repo's incumbent UI tech.
   NativeScript has two stacked breaks (executable-memory/JIT plus
  runtime-dex/storage-DCL),
   its only app-side mitigation (`--jitless`) does not work,
   and the gateway has no
  Rust or idiomatic-Kotlin home (it forces Kotlin/Java under a JS shell that adds nothing).

Flip conditions:
 if iOS or desktop becomes a real target,
 Compose Multiplatform rises above native Jetpack;
if keeping the gateway in Rust and out of the JVM is weighted highest,
 Tauri rises above native Jetpack;
 if
the target were a device without DCL hardening (stock Android,
 or GrapheneOS with the memory and storage DCL
toggles off for this app),
 NativeScript would boot and rise above Slint,
 but its gateway still has no Rust home
(forced into Kotlin with no monorepo reuse) and it carries a second JS engine,
 so it would sit below Tauri
(which reuses the repo's axum and reqwest crates),
 around the Compose Multiplatform tier;
 on the owner's
hardened Pixel 6 it is last.

The pick is a value judgment reserved to the owner;
 this document records the comparison,
 not a selection.

## Per-technology scorecard

S3-gateway core,
 Rust path (Tauri and Slint):
 axum 0.8.9 plus reqwest 0.13.4 streamed a 256 MiB object
end to end at about 8.2 MiB peak RSS,
 with pure-Rust TLS (rustls,
 no OpenSSL).

S3-gateway core,
 Kotlin path (Jetpack and Compose Multiplatform):
 Ktor CIO server verified in-app on the
device;
 OkHttp 5.3.2 / Retrofit 3.0.0 streaming client verified.
 Rejected:
 NanoHTTPD (dormant,
 blocking),
Spring Boot (too heavy),
 `java.net.http` (no Ktor-engine path).

S3-gateway core,
 NativeScript path:
 no native home.
 `@nativescript/core` HTTP is a buffering client with no
server and no streaming (`Async.java:398` writes the whole body;
 an in-source TODO admits large files will not
work),
 and the JS to Java bridge copies buffers per crossing (`index.android.ts:158-160`),
 so the streaming
pump cannot live in JS.
 The viable hosts are `java.net.ServerSocket`,
 NanoHTTPD (unmaintained since 2019),
 or
Ktor CIO,
 all driven from `App_Resources/Android` and pumping in Kotlin/Java,
 identical to the Kotlin-path
gateway above but wrapped in a JS shell.
 The one JS-resident option,
 nodejs-mobile,
 means a second JS engine
(`libnode.so` about 62 MiB per ABI) and its NativeScript binding is gone from npm.
 Moot on the target device:
V8 does not initialize (see below).

V8 executable-memory disqualifier (NativeScript):
 on the owner's GrapheneOS Pixel 6,
 V8 cannot allocate
executable memory to deserialize its startup snapshot (`TSEC_FLAG_DENY_EXECMEM`,
 the memory-DCL protection),
so it aborts at isolate init before any app JS.
 Verified on device for both the default JIT build and a
`--jitless` build (identical crash).
 This is the executable-memory side of GrapheneOS DCL,
 distinct from
Slint's `InMemoryDexClassLoader` dex-loading break.

Background execution (needed by every stack):
 WorkManager plus a dataSync Foreground Service verified
surviving backgrounding on the device.
 Constraint:
 apps targeting API 35+ get about 6 hours of dataSync
foreground-service time per 24 hours,
 after which `onTimeout` fires.
 A kopia snapshot can exceed that,
 so
backups must be chunked and resumable with onTimeout checkpointing,
 and the bulk transfer should use a
user-initiated data transfer job rather than a bare dataSync service.

Testing,
 in-process:
 runComposeUiTest (Compose Multiplatform,
 verified,
 offscreen,
 deterministic clock;
adopt the `v2` entry point since the classic one is deprecated in 1.12-alpha);
 createAndroidComposeRule
(native Jetpack,
 verified;
 pin androidx.
test 3.7.0 / runner 1.7.0 / core 1.7.0 on Android 15/16);
slint:
:
testing (verified headless,
 but text input is gated);
 tauri-driver (verified on Linux desktop but
pre-alpha,
 Linux and Windows only,
 no native clicks).

Testing,
 black-box end-to-end (the Playwright peer):
 Maestro verified driving native Compose via testTag
(`Modifier.semantics { testTagsAsResourceId = true }`),
 single binary,
 self-cleaning;
 the recommended
peer for native Android.
 Appium plus the uiautomator2 driver also verified (W3C,
 heavier;
 locate via
`-android uiautomator` or accessibility-id).
 UiAutomator is the shared primitive both wrap.

Fuzzing:
 Jazzer (`com.code-intelligence:jazzer`,
 on Maven Central,
 JUnit5 `@FuzzTest`) verified,
 found a
planted crash in a Kotlin parser in under a second and emitted a reproducer.
 It is the JVM fuzzer to use.
kotlinx.
fuzz is disqualified:
 uninstallable (its only artifact host is NXDOMAIN,
 not on Maven Central,
plugin-portal publish disabled),
 and it is a research-lab project,
 not an official Kotlin library.
 For the
Rust path,
 cargo-fuzz (the repo incumbent) is verified.

Mutation testing:
 Pitest works on Kotlin (a sample scored 16 mutations,
 12 killed) but its open-source
Kotlin support is a self-described "quick dirty hack" (`KotlinFilter.java:14-52`),
 with proper handling
paywalled behind the commercial Arcmutate plugin.
 For the Rust path,
 cargo-mutants 27.
x is verified and is
effectively the only maintained Rust option.
 Mutation testing is optional either way.

Property testing:
 Kotest (verified,
 with shrinking) on the Kotlin path;
 proptest 1.
x (verified,
 integrated
shrinking) on the Rust path.
 kotlin.
test is the minimal Kotlin assertion layer (verified),
 pair it with
JUnit5.

## Cross-cutting Android constraints (any stack)

- GrapheneOS device:
   INTERNET is a revocable per-app permission (default off),
   so the app must request
  Network access;
   dynamic code loading is restricted on two axes,
   both enforced on this device.
   Dex loading
  (memory and storage) raises a `SecurityException` at `DexClassLoader`/`InMemoryDexClassLoader` (this is what
  disqualifies Slint's UI,
   and is the second,
   latent break for any NativeScript app that subclasses a native
  type at runtime).
   Executable memory / app JIT is denied separately (`TSEC_FLAG_DENY_EXECMEM`),
   which
  disqualifies any stack embedding its own JIT engine:
   this is what kills NativeScript's V8 at startup,
   and
  NativeScript applies its `v8Flags` too late (after `Isolate::New`) for the `--jitless` flag to avoid it.
  Stacks that emit no runtime dex and embed no JIT (native Kotlin in
  `classes.dex`,
   Tauri's system WebView,
   Rust) clear both axes.
- Foreground-service time cap (API 35+):
   about 6 hours of dataSync per 24 hours;
   design backups as
  chunked,
   resumable,
   onTimeout-checkpointed,
   and prefer a user-initiated data transfer job for bulk.
- kopia binary:
   ship per ABI as `jniLibs/<abi>/lib*.so` with `extractNativeLibs=true`,
   and exec it from
  `nativeLibraryDir` (W^X since API 29).
   This is language-neutral.
- Native shared objects need 16 KB page alignment on Android 15/16 (hit during the Slint build,
   fixed with
  `-Wl,-z,max-page-size=16384`);
   relevant to the kopia `.so` and any Rust `.so`.

## Evidence

Seventeen per-technology full-verification vets were run (sixteen on 2026-06-07,
 plus NativeScript on
2026-06-12),
 each producing a report with exact commands,
 outputs,
 and source citations.
 The load-bearing
numbers and version pins are inlined above so this document stands alone.

The evidence paths spelled `kotlin-android-kopia-pcloud-vet-reports/` when this record was updated on 2026-06-12.
The folder became `kotlin-android-kopia-pcloud-vet-report/` on 2026-07-15 (`bd19d57e20`),
so the `kotlin-android-kopia-pcloud-vet-reports/` spelling no longer resolves.

 The full raw reports are persisted
alongside this doc under `kotlin-android-kopia-pcloud-vet-report/` (verbatim agent output,
 not
lint-conformed):

- Stacks:
   `vet-jetpack-compose.md`,
   `vet-compose.md`,
   `vet-tauri.md`,
   `vet-slint-rust.md`,
   `vet-nativescript.md`.
- Gateway core:
   `vet-ktor.md`,
   `vet-okhttp.md`,
   `vet-rust-http-core.md`.
- Background and runtime:
   `vet-android-runtime.md`.
- Testing:
   `vet-slint-testing.md`,
   `vet-tauri-driver.md`,
   `vet-android-e2e.md`,
   `vet-test-frameworks.md`.
- Fuzz and mutation:
   `vet-jazzer.md`,
   `vet-kotlinx-fuzz.md`,
   `vet-pitest.md`,
   `vet-rust-qa.md`.

## Out of scope (owner-decided)

Running kopia on Android,
 the pCloud native API,
 and whether the custom S3 gateway is necessary are
product and architecture decisions made by the owner and are not assessed here.
