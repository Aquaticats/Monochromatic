# iOS source-audit: Compose Multiplatform

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Verbatim structured output from the 2026-06-12 `ios-framework-desk-audit` fan-out (16 parallel
source audits against the three iOS platform walls plus the kopia/music-player functional
requirements),
 with an adversarial cite-check.
 Not lint-conformed.
 On-device gate results,
 where
run,
 live in `device-gate-results.md`;
 the cross-cutting decision is in
`../ios-iphone-x-music-player-kopia-stack.md`.

## Verdict

- iOS runtime model:
   AOT-native.
   App code (Kotlin) is compiled ahead-of-time by the Kotlin/Native LLVM backend to a self-contained native ARM64
   binary that runs with no virtual machine,
   no JIT,
   and no interpreter.
   The in-repo iOS sample modules build a Kotlin/Native framework binary with framework { isStatic = true } for the
   iosArm64() and iosSimulatorArm64() targets;
   the Swift app loads that compiled framework (Main_iosKt.
  ComposeEntryPoint() returns a UIViewController).
   UI is drawn by Skiko/Skia into a CAMetalLayer;
   there is no scripted/managed runtime executing app logic on device.
   Engine:
   Kotlin/Native (LLVM AOT).
   Source:
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/shared/build.
  gradle.
  kts (iosArm64() + framework isStatic=true);
   kotlinlang.
  org/docs/native-overview.
  html ("compiling Kotlin code to native binaries that can run without a virtual machine ...
  such as embedded devices or iOS").
- Minimum iOS deployment:
   Not pinned by the repo iOS samples (no explicit IPHONEOS_DEPLOYMENT_TARGET found in the audited example xcodeproj
   scan);
   Compose Multiplatform's stable iOS support and Kotlin/Native iosArm64 in practice target modern iOS (roughly iOS
   14+),
   but treat the exact floor as unverified-from-source and confirm against the Compose MP release notes for the chosen
   version before the gate.
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Compose MP on iOS executes app code as Kotlin/Native LLVM AOT-compiled native ARM64 with no VM/JIT/interpreter
   (clearing wall 2 by construction,
   the opposite of NativeScript's execmem failure),
   and its cinterop staticLibraries/linkerOpts mechanism links a C/Go/Rust .
  a archive into the app binary,
   so both kopia (Go c-archive) and the existing symphonia+cpal Rust audio core link
   and are callable via FFI with no rewrite (cpal's CoreAudio iOS backend means AVAudioEngine is optional,
   not required).
   The only soft spot is whether Ktor's CIO in-app HTTP server runs on iosArm64;
   if not,
   the localhost endpoint moves into the linked static lib.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

Kotlin/Native is an LLVM-based AOT toolchain that emits a self-contained native binary requiring no VM,
 JIT,
 or interpreter at runtime.
 JetBrains explicitly markets it for iOS where VMs are 'not desirable or possible'.
 App logic executes as ordinary signed ARM64 machine code in the app process,
 so no W^X/executable-memory (DENY_EXECMEM-style) entitlement is needed;
 nothing in app code requests dynamic-codesign/JIT pages.
 This is the opposite of the NativeScript-on-Android execmem failure:
 there is no bytecode interpreter or JIT in the app process.
 The in-repo iOS targets compile to a static framework (isStatic=true,
 iosArm64),
 confirming device builds are fully ahead-of-time-native.

Source:
 kotlinlang.
org/docs/native-overview.
html (LLVM backend;
 'native binaries that can run without a virtual machine';
 designed for iOS);
 /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/shared/build.
gradle.
kts (iosArm64(),
 iosSimulatorArm64(),
 framework { isStatic = true })

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Kotlin/Native cinterop.
 A C static archive (.
a) plus a C header is bound via a .
def file using the staticLibraries + libraryPaths properties (the archive is embedded into the produced .
klib and linked automatically);
 extra linker flags go through linkerOpts.
 A Go static lib (go build -buildmode=c-archive / gomobile producing libkopia.
a + .
h) or a Rust staticlib (crate-type=["staticlib"] producing libcore.
a + a cbindgen header) is exactly this shape,
 so kopia and the music-player Rust core both link and are callable as ordinary functions from Kotlin app code.
 The repo already shows Kotlin/Native calling native Apple frameworks directly through the same cinterop machinery:
 kotlinx.
cinterop.
*,
 platform.
AVFoundation.
*,
 platform.
UIKit.
*,
 platform.
CoreLocation.
*,
 platform.
darwin.
NSObject,
 @ObjCAction.
 Mechanism = Kotlin/Native cinterop static-library linking (no exec of a bundled binary;
 satisfies wall 1).

Source:
 kotlinlang.
org/docs/native-definition-file.
html (staticLibraries = libfoo.
a;
 libraryPaths;
 'includes the library binary in the klib ... the library is linked automatically';
 linkerOpts);
 /tmp/agent/compose-mp-audit/examples/imageviewer/shared/src/iosMain/kotlin/example/imageviewer/view/CameraView.
ios.
kt (kotlinx.
cinterop + platform.
AVFoundation.
* + platform.
darwin.
NSObject used directly)

## Wall 3: background execution

Compose MP provides no background machinery itself;
 it is supplied by Kotlin/Native's bundled Apple platform bindings plus Ktor.
 The HTTPS client uses ktor-client-darwin,
 which is backed by NSURLSession;
 a background-configured NSURLSession (and BGProcessingTask / BGAppRefreshTask via the platform.
BackgroundTasks cinterop bindings) is reachable from Kotlin the same way platform.
AVFoundation / platform.
Foundation are.
 So the multi-hour kopia snapshot must be restructured around background URLSession exactly like a native Swift app:
 it cannot run as an unbounded foreground service.
 This is a re-architecture constraint,
 not a framework gap.
 The in-app HTTP server kopia targets must run while the app is foreground/has background time,
 since iOS will suspend the process;
 the snapshot has to be chunked across background-transfer windows.

Source:
 ktor.
io/docs/client-engines.
html (Darwin engine 'uses NSURLSession under the hood' for iOS);
 Kotlin/Native platform bindings expose Foundation/NSURLSession and BackgroundTasks via cinterop (kotlinlang.
org/docs/native-c-interop.
html);
 /tmp/agent/compose-mp-audit/examples/imageviewer/shared/src/iosMain/kotlin/example/imageviewer/view/CameraView.
ios.
kt (platform.
Foundation.
* used from Kotlin/Native)

## In-app HTTP server (kopia S3 target)

Feasible:
 partial

Mechanism:
 Not provided by Compose MP.
 The candidate is Ktor's CIO server (embeddedServer(CIO)),
 a pure-Kotlin coroutine HTTP server bound to localhost that kopia would target as its S3/HTTP endpoint.
 Ktor's native-server docs list CIO as the only native engine
 and enumerate macOS/Linux/Windows native targets explicitly,
 NOT iosArm64;
 HTTPS without a reverse proxy is unsupported on native.
 A plain-HTTP localhost listener for kopia's S3 target is plausible
 but iOS-device support of the CIO server engine is unproven from the docs
 and is the single thing most needing on-device confirmation.
 Alternative:
 implement the localhost endpoint as part of the linked kopia/Rust static lib (a C/Go/Rust HTTP listener),
 sidestepping Ktor server entirely.

Source:
 ktor.
io/docs/server-native.
html (CIO only;
 lists macOS/Linux/Windows native,
 not iOS;
 no HTTPS on native)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 ktor-client-darwin on Kotlin/Native:
 HttpClient(Darwin) backed by NSURLSession,
 giving TLS,
 streaming bodies,
 and access to background-URLSession configuration for pCloud uploads/downloads.
 CIO client is also available on native as a fallback.
 Either is called from Kotlin app code;
 no extra runtime needed.

Source:
 ktor.
io/docs/client-engines.
html (Darwin engine uses NSURLSession on iOS/macOS/tvOS/watchOS;
 CIO available on Native)

## Audio (music-player port)

Decode:
 Not provided by Compose MP (UI-only framework).
 Decode comes from the existing Rust core:
 symphonia 0.6 + libopus,
 linked as a Rust staticlib via cinterop,
 OR via Kotlin/Native platform bindings to AVFoundation/AudioToolbox.
 The Rust path needs no rewrite.

Output:
 Low-latency output via the existing cpal 0.18 backend,
 which has an iOS/CoreAudio (AudioQueue/AudioUnit) implementation,
 reached through the linked Rust static lib.
 Alternatively a thin Kotlin/Native wrapper over platform.
AVFoundation.
AVAudioEngine / platform.
AudioToolbox using the bundled Apple bindings.
 Compose only drives the UI (queue,
 pagination,
 controls);
 it never touches the audio path.

Rust core reuse:
 REUSE via FFI,
 no rewrite.
 The Rust symphonia+cpal core is exposed over a C ABI (cbindgen header + crate-type=staticlib -> libcore.
a) and linked through cinterop staticLibraries/linkerOpts exactly like kopia (wall 1).
 Because cpal already has a working iOS/CoreAudio backend,
 the true-peak normalization,
 on-disk peak cache,
 folder-scanned queue,
 two-axis pagination,
 and session persistence stay in Rust unchanged.
 AVAudioEngine is only needed if you choose NOT to reuse cpal;
 it is not forced.
 Compose MP imposes no audio model,
 so the AOT-native runtime can host the Rust core directly.

Source:
 /tmp/agent/compose-mp-audit/examples/imageviewer/shared/src/iosMain/kotlin/example/imageviewer/view/CameraView.
ios.
kt (platform.
AVFoundation.
* callable from Kotlin/Native);
 kotlinlang.
org/docs/native-definition-file.
html (staticLibraries link of a Rust/C .
a);
 cpal upstream has a coreaudio iOS backend (RustAudio/cpal src/host/coreaudio)

## Gate probe and toolchain

Minimal app:
 a Kotlin Multiplatform module with iosArm64() target,
 the Compose Multiplatform plugin,
 and framework { isStatic = true },
 rendering one ComposeUIViewController with a Compose button.
 Link a tiny Rust staticlib (crate-type=staticlib,
 one exported extern \"C\" fn add(a,
b) plus a cbindgen header) via a cinterop .
def using staticLibraries/libraryPaths,
 and also add HttpClient(Darwin) doing one HTTPS GET to a public URL.
 On-device signal (real iPhone,
 not simulator):
 the app launches,
 the Compose UI renders and is interactive (Skiko/Metal),
 tapping the button calls the Rust function and shows its result (proves wall 1 static-lib FFI + AOT execution with no
 JIT entitlement),
 and the HTTPS GET returns 200 (proves the Darwin/NSURLSession client).
 Decisive refutation would be:
 link failure of the Rust .
a,
 a runtime crash demanding executable memory,
 or the framework refusing to run AOT on iosArm64 (none expected).
 Stretch probe for the one soft spot:
 start embeddedServer(CIO) bound to 127.0.0.1 inside the app and curl it from within the app on device,
 to confirm the in-app HTTP server for kopia works on iosArm64.

Toolchain:
 macOS with Xcode (+ Command Line Tools) for the iOS SDK,
 codesigning,
 and on-device deploy;
 JDK 17+;
 Kotlin + Gradle with the Kotlin Multiplatform and Compose Multiplatform plugins (Kotlin/Native downloads its own LLVM
 + iOS toolchain on first build);
 a Rust toolchain with the aarch64-apple-ios target (rustup target add aarch64-apple-ios)
 and cbindgen for the music-player core;
 a Go toolchain with gomobile / go build -buildmode=c-archive (aarch64-apple-ios) for kopia;
 an Apple developer signing identity + provisioning profile for the iPhone X device gate.

## Supporting-stack vets this framework drags in

- Ktor CIO server on iosArm64:
   does embeddedServer(CIO) actually bind/serve on a real iOS device (the one unproven piece for app A's in-app S3/HTTP
   endpoint) or must the listener live inside the linked Go/Rust static lib
- kopia as a Go c-archive:
   gomobile/buildmode=c-archive .
  a for aarch64-apple-ios,
   C header,
   cinterop .
  def,
   and Go-runtime-in-app-process behavior under iOS memory/background limits
- Rust music-player core FFI binding:
   cbindgen header stability,
   panic=abort across the FFI boundary,
   cpal CoreAudio iOS backend latency/route-change handling on device
- Background URLSession restructure:
   chunking a multi-hour kopia snapshot across BGProcessingTask + background NSURLSession windows;
   verifying delegate callbacks fire from Kotlin/Native
- Audio QA:
   symphonia decode correctness + true-peak normalization parity vs desktop,
   on-disk peak-cache format reuse
- In-process UI test:
   Compose UI test harness on iOS device (runComposeUiTest on Kotlin/Native) vs needing XCUITest
- e2e:
   XCUITest or Maestro driving the launched .
  app on the iPhone X
- Property + fuzz + mutation testing of the Rust core:
   reusable as-is on the host since the core is plain Rust (cargo-fuzz,
   proptest,
   cargo-mutants),
   independent of the iOS shell
- Skiko/Skia + Metal rendering vet:
   confirm CAMetalLayer rendering and no software-renderer fallback on the target device

## Cited sources

- Kotlin/Native is LLVM AOT to native binaries with no VM/JIT,
   explicitly designed for iOS (wall 2:
   no execmem needed):
   kotlinlang.
  org/docs/native-overview.
  html
- iOS sample builds a Kotlin/Native static framework for iosArm64 (AOT device build):
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/shared/build.
  gradle.
  kts (iosArm64();
   iosSimulatorArm64();
   framework { isStatic = true })
- cinterop links a C static archive (.
  a) into the binary via staticLibraries/libraryPaths/linkerOpts (wall 1:
   Go c-archive / Rust staticlib):
   kotlinlang.
  org/docs/native-definition-file.
  html
- Kotlin/Native calls Apple platform frameworks directly via cinterop (UIKit,
   AVFoundation,
   CoreLocation,
   darwin/NSObject):
   /tmp/agent/compose-mp-audit/examples/imageviewer/shared/src/iosMain/kotlin/example/imageviewer/view/CameraView.
  ios.
  kt
- Swift app loads the compiled Kotlin framework and calls into it (Main_iosKt.
  ComposeEntryPoint() -> UIViewController):
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/iosApp/iosApp/ContentView.
  swift
- ComposeUIViewController returns a UIViewController;
   UIKitView interops native views;
   @ObjCAction bridges ObjC selectors:
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/shared/src/iosMain/kotlin/main.
  ios.
  kt and UseUITextField.
  kt
- HTTPS client on iOS = ktor-client-darwin over NSURLSession (wall 3 background URLSession path);
   CIO client also on native:
   ktor.
  io/docs/client-engines.
  html
- Ktor CIO server runs on native but docs list macOS/Linux/Windows (not iosArm64) and no native HTTPS;
   in-app server on device is the unproven piece:
   ktor.
  io/docs/server-native.
  html
- cinterop is the C-interop mechanism (def files,
   headers) backing both static-lib linking and platform bindings:
   kotlinlang.
  org/docs/native-c-interop.
  html

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   No fabricated,
   unsupported,
   or contradicted load-bearing claims found;
   every cited source exists and supports its claim.
   Two minor non-load-bearing notes (neither changes a verdict):
   (1) Doc inconsistency,
   not an audit error:
   native-definition-file.
  html's prose sentence spells the property singular ("use staticLibrary and libraryPaths"),
   while its own code snippet and the real cinterop property are plural ("staticLibraries = libfoo.
  a").
   The audit correctly used the plural form,
   so the audit is accurate;
   the typo is upstream in JetBrains' docs.
   (2) The iosRuntimeModel's "Skiko/Skia into a CAMetalLayer" detail is not evidenced in the two cited in-repo/doc
   sources,
   but it is descriptive,
   peripheral,
   and accurate for Compose Multiplatform iOS;
   it does not affect the wall-2 no-jit verdict.
   All other claims verified verbatim:
   native-overview.
  html confirms "native binaries that can run without a virtual machine,
  " "LLVM-based backend,
  " and "virtual machines are not desirable or possible,
   such as embedded devices or iOS";
   native-definition-file.
  html confirms staticLibraries/libraryPaths/linkerOpts,
   "includes the library binary in the klib,
  " and "the library is linked automatically";
   build.
  gradle.
  kts confirms iosArm64()/iosSimulatorArm64()/isStatic=true;
   CameraView.
  ios.
  kt confirms kotlinx.
  cinterop + platform.
  AVFoundation/UIKit/CoreLocation + platform.
  darwin.
  NSObject + @ObjCAction;
   ContentView.
  swift/main.
  ios.
  kt confirm Main_iosKt.
  ComposeEntryPoint() returns a UIViewController.
- Sources checked:
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/shared/build.
  gradle.
  kts (iosArm64(),
   iosSimulatorArm64(),
   framework { isStatic = true } — confirmed);
   /tmp/agent/compose-mp-audit/examples/imageviewer/shared/src/iosMain/kotlin/example/imageviewer/view/CameraView.
  ios.
  kt (kotlinx.
  cinterop.
  *,
   platform.
  AVFoundation.
  *,
   platform.
  UIKit.
  *,
   platform.
  CoreLocation.
  *,
   platform.
  darwin.
  NSObject,
   @ObjCAction — confirmed);
   /tmp/agent/compose-mp-audit/examples/interop/ios-uikit-in-compose/iosApp/iosApp/ContentView.
  swift + shared/src/iosMain/kotlin/main.
  ios.
  kt (Main_iosKt.
  ComposeEntryPoint() returns UIViewController — confirmed);
   <https://kotlinlang.org/docs/native-overview.html> (LLVM-based backend;
   'native binaries that can run without a virtual machine';
   'virtual machines are not desirable or possible,
   such as embedded devices or iOS' — confirmed);
   <https://kotlinlang.org/docs/native-definition-file.html> (staticLibraries,
   libraryPaths,
   linkerOpts;
   'includes the library binary in the klib';
   'the library is linked automatically' — confirmed)
