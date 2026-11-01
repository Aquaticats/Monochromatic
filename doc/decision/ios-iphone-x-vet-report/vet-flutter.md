# iOS source-audit: Flutter

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
   On an iOS device in profile/release,
   Dart app code is compiled ahead-of-time into a native arm64 snapshot shipped inside App.
  framework (a precompiled dylib),
   and the engine links the no-JIT Dart runtime (libdart_aotruntime).
   The engine probes Dart_IsPrecompiledRuntime() and,
   when true,
   registers App.
  framework's executablePath as the application library;
   no kernel/JIT snapshot is loaded.
   Only the DEBUG build links libdart_jit and loads a kernel snapshot (the JIT path
    that needs the dynamic-codesigning entitlement and is never used for App Store / release on device).
   Engine named:
   Flutter Engine + Dart VM in AOT (precompiled-runtime) mode,
   rendering via Impeller/Metal.
- Minimum iOS deployment:
   iOS 13 (Flutter supported_platforms:
   iOS 13 to 26;
   FFI plugin podspec sets s.
  platform = :
  ios,
   '13.0')
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Flutter runs iOS release app code as ahead-of-time-compiled native arm64 (libdart_aotruntime,
   App.
  framework precompiled snapshot,
   libdart_jit only in debug),
   so it clears the iOS no-JIT/no-execmem wall outright;
   and dart:
  ffi statically links and calls a Go/Rust c-archive via DynamicLibrary.
  executable,
   meaning both kopia and the existing symphonia+cpal music-player Rust core are reusable behind it
    with no AVAudioEngine rewrite (cpal already has a RemoteIO iOS backend).
   Only the GUI (Slint) is rewritten,
   not the native cores.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

Flutter release/profile on iOS uses Dart AOT:
 the app's Dart is precompiled to a native snapshot in App.
framework and the engine links libdart_aotruntime (no JIT,
 no writable+executable memory for app code).
 runtime/BUILD.
gn group("libdart") selects $dart_src/runtime:
libdart_aotruntime for flutter_runtime_mode == profile|release,
 and only the else branch (debug) links libdart_jit + //flutter/lib/snapshot.
 DartVM:
:
IsRunningPrecompiledCode() returns Dart_IsPrecompiledRuntime();
 FlutterDartProject.
mm guards App.
framework executable loading on that being true,
 and only loads a kernel snapshot when it is false (debug).
 So app-process code never needs the WKWebView-only JIT entitlement;
 this clears the iOS no-execmem wall the same way it would clear GrapheneOS DENY_EXECMEM.

Source:
 /tmp/agent/flutter-audit-20260612/engine/src/flutter/runtime/BUILD.
gn:
31-38 (libdart_aotruntime for profile/release vs libdart_jit for debug);
 engine/src/flutter/runtime/dart_vm.
cc:
177-179 (IsRunningPrecompiledCode -> Dart_IsPrecompiledRuntime);
 engine/src/flutter/shell/platform/darwin/ios/framework/Source/FlutterDartProject.
mm:
103-160 (AOT App.
framework executable path vs debug-only kernel snapshot)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 dart:
ffi.
 A Go gomobile c-archive (.
a) or Rust staticlib (crate-type=staticlib,
 .
a) is statically linked into the app binary via the FFI-plugin CocoaPods/Xcode build,
 and its C-ABI symbols are resolved at runtime with DynamicLibrary.
executable()/DynamicLibrary.
process() (no dlopen needed for statically-linked symbols;
 dynamic libs use DynamicLibrary.
open with a .
framework).
 Bindings are generated from the C header by package:
ffigen.
 dart:
ffi is a mapped core library in the Flutter iOS embedder.
 Long-running native calls (a kopia snapshot,
 audio decode) run on a helper Dart isolate so they don't block the UI thread,
 per the FFI plugin template.
 This is the direct,
 supported path to call kopia-as-linked-static-lib from app code (wall 1 satisfied).

Source:
 /tmp/agent/flutter-audit-20260612/engine/src/flutter/sky/packages/sky_engine/lib/`_embedder.yaml`:
10 (dart:
ffi mapped);
 package/flutter_tools/templates/plugin_ffi/ (ffiPlugin scaffold:
 ios.
tmpl/projectName.
podspec.
tmpl source_files static-linked into app,
 `README.md`.
tmpl ffigen + helper-isolate guidance);
 docs.
flutter.
dev/platform-integration/ios/c-interop (static link -> DynamicLibrary.
executable/process)

## Wall 3: background execution

No long-running background service exists on iOS,
 same constraint as every candidate.
 Flutter expresses background transfer
  through native URLSession via mature plugins (background_downloader uses iOS URLSessions;
 background_transfer uses NSURLSession + registers a BGTaskScheduler/BGProcessingTask identifier in AppDelegate
  for iOS 15+).
 A multi-hour kopia snapshot cannot run as a foreground service;
 it must be restructured around background URLSession chunked uploads,
 or driven while the app is foreground/charging via BGProcessingTask windows.
 Flutter does not lift this OS limit;
 it just surfaces the same URLSession/BGProcessingTask primitives through plugins or a platform channel to custom Swift.

Source:
 docs.
flutter.
dev/packages-and-plugins/background-processes;
 pub.
dev/packages/background_downloader (URLSession on iOS);
 pub.
dev/packages/background_transfer (NSURLSession + BGTaskScheduler registration in AppDelegate,
 iOS 15+)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 dart:
io HttpServer.
bind() runs an in-process HTTP/1.1 server entirely in Dart over the Dart VM socket layer on the loopback interface,
 no bundled-binary exec required.
 This gives kopia an in-app HTTP/S3-style endpoint to target (kopia repository server
  or an S3-compatible shim implemented over HttpServer,
 or fronting the linked kopia static lib).
 dart:
io is a mapped core library in the Flutter iOS embedder,
 so it is available on device with the AOT runtime.

Source:
 /tmp/agent/flutter-audit-20260612/engine/src/flutter/sky/packages/sky_engine/lib/`_embedder.yaml`:
12 (dart:
io mapped to third_party/dart/sdk/lib/io/io.
dart);
 dart:
io HttpServer is standard Dart SDK API

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 dart:
io HttpClient (and package:
http / package:
dio on top of it) provides a streaming HTTPS client with chunked request/response bodies suitable for streaming uploads
 to pCloud;
 for truly background/resumable transfers that survive app suspension,
 route through URLSession via background_downloader.
 TLS is handled by the Dart VM's BoringSSL-backed SecureSocket,
 no exec.
 dart:
io is mapped in the Flutter iOS embedder.

Source:
 /tmp/agent/flutter-audit-20260612/engine/src/flutter/sky/packages/sky_engine/lib/`_embedder.yaml`:
12 (dart:
io mapped);
 pub.
dev/packages/background_downloader for background URLSession uploads

## Audio (music-player port)

Decode:
 symphonia 0.6 (pure-Rust,
 no exec,
 no JIT) plus libopus stays in the existing Rust core and is called over dart:
ffi;
 Flutter itself supplies no decoder for this path.

Output:
 Low-latency output via cpal's iOS CoreAudio backend,
 which drives a RemoteIO AudioUnit (kAudioOutputUnitProperty_EnableIO,
 AudioComponent RemoteIO) configured through AVAudioSession,
 buffer sizes 256-4096 frames.
 This is direct AudioUnit output,
 not AVAudioEngine,
 so true-peak normalization and the peak cache remain in Rust.

Rust core reuse:
 REUSED via FFI,
 no AVAudioEngine rewrite.
 cpal 0.18 already ships an iOS CoreAudio (RemoteIO AudioUnit) backend,
 and symphonia+libopus are pure Rust.
 The whole package/music-player/desktop-app Rust core (symphonia decode,
 true-peak normalization,
 on-disk peak cache,
 two-axis paginated folder queue,
 session persistence,
 cpal output) compiles to a Rust staticlib,
 links into the iOS app,
 and is called from Dart via dart:
ffi exactly as kopia is.
 The Slint UI layer is replaced by Flutter widgets;
 only the GUI is rewritten,
 not the audio pipeline.
 Caveat to verify on device:
 cpal's RemoteIO stream must run while the AVAudioSession category is configured for playback,
 and background audio needs the UIBackgroundModes audio entitlement;
 the Rust audio thread itself is fine under AOT (native code,
 no JIT).

Source:
 RustAudio/cpal src/host/coreaudio/ios/`mod.rs` (RemoteIO AudioUnit + AVAudioSession output backend);
 symphonia is pure Rust

## Gate probe and toolchain

Minimal app:
 flutter create --template=plugin_ffi a tiny FFI plugin wrapping a Rust staticlib (crate-type=[\"staticlib\"])
  that exports one extern \"C\" fn (e.g. add(a,
b) and a 1-second sine-into-cpal play call),
 build a release (AOT) .
app for a real device,
 sign,
 install on the iPhone X,
 and from Dart resolve the symbol with DynamicLibrary.
executable() and call it.
 On-device signals that confirm viability:
 (1) the release build runs at all (proves AOT snapshot in App.
framework executes with libdart_aotruntime,
 wall 2);
 (2) the FFI call returns the correct value (proves Rust staticlib statically linked and callable,
 wall 1);
 (3) the cpal RemoteIO stream produces audible output (proves symphonia+cpal core reusable
  for the music-player without AVAudioEngine).
 Refutation signals:
 release build crashes on launch with a codesign/execmem fault (would mean JIT leaked into release,
 contradicting the BUILD.
gn evidence),
 or the FFI symbol fails to resolve (static-link path broken).

Toolchain:
 Xcode (+ command-line tools) with a real device deployment target iOS 13+,
 CocoaPods;
 Flutter SDK (flutter,
 dart) on macOS;
 Rust toolchain with the aarch64-apple-ios target (rustup target add aarch64-apple-ios) for the staticlib,
 or Go with gomobile bind -target=ios for a Go c-archive .
xcframework;
 ffigen (dart run ffigen) for bindings;
 an Apple developer signing identity / provisioning profile for on-device install.

## Supporting-stack vets this framework drags in

- dart:
  ffi binding vet:
   ffigen-generated bindings to the kopia Go c-archive and the music-player Rust staticlib,
   including helper-isolate offloading and FFI callbacks/NativePort for progress
- In-app HTTP/S3 endpoint vet:
   dart:
  io HttpServer (or a Dart S3 shim) fronting the linked kopia static lib,
   loopback-only binding
- HTTPS streaming-upload vet:
   dart:
  io HttpClient / package:
  dio streaming bodies to pCloud,
   plus background_downloader URLSession path
- Background-transfer vet:
   background_downloader / custom platform-channel Swift wiring BGProcessingTask + background URLSession,
   restructuring multi-hour snapshots into resumable chunked uploads
- Audio vet:
   cpal 0.18 RemoteIO/AVAudioSession output on device,
   AVAudioSession category + UIBackgroundModes audio for background playback,
   low-latency buffer tuning,
   true-peak normalization parity with desktop
- Platform-channel vet:
   MethodChannel/Pigeon for any iOS-only native (keychain,
   BGTaskScheduler registration,
   AVAudioSession setup)
- QA:
   in-process widget tests (flutter_test) for UI;
   integration_test driving the on-device app;
   e2e via flutter drive;
   property/fuzz tests on the Rust core (cargo fuzz,
   proptest) since the audio/normalization logic stays in Rust;
   Dart-side property tests;
   mutation testing is weak in the Dart ecosystem (no mature mutation tool),
   so coverage leans on the Rust core's own test suite

## Cited sources

- iOS profile/release link the AOT (no-JIT) Dart runtime;
   only debug links the JIT runtime:
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/runtime/BUILD.
  gn:
  31-38
- DartVM:
  :
  IsRunningPrecompiledCode() == Dart_IsPrecompiledRuntime();
   release uses precompiled (AOT) snapshot:
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/runtime/dart_vm.
  cc:
  177-179
- Engine loads App.
  framework precompiled executable when running precompiled code;
   loads kernel/JIT snapshot only when NOT precompiled (debug):
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/shell/platform/darwin/ios/framework/Source/FlutterDartProject.
  mm:
  103-160,
   240-249
- dart:
  ffi and dart:
  io are first-class mapped libraries in the Flutter iOS embedder:
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/sky/packages/sky_engine/lib/`_embedder.yaml`:
  10,12
- FFI plugin scaffold statically links native source into the iOS app via podspec source_files and binds via ffigen,
   with a helper-isolate pattern for long-running native calls:
   /tmp/agent/flutter-audit-20260612/packages/flutter_tools/templates/plugin_ffi/ios.
  tmpl/projectName.
  podspec.
  tmpl and `README.md`.
  tmpl
- Statically linked symbols are resolved with DynamicLibrary.
  executable()/process();
   both static and dynamic linking supported on iOS:
   <https://docs.flutter.dev/platform-integration/ios/c-interop>
- Minimum supported iOS is 13 (FFI plugin podspec ios 13.0):
   <https://docs.flutter.dev/reference/supported-platforms> and plugin_ffi ios.
  tmpl podspec
- Background transfer on iOS is URLSession/BGProcessingTask via plugins,
   no long-running service:
   <https://docs.flutter.dev/packages-and-plugins/background-processes> ;
   <https://pub.dev/packages/background_downloader> ;
   <https://pub.dev/packages/background_transfer>
- cpal has an iOS CoreAudio output backend using a RemoteIO AudioUnit + AVAudioSession (not AVAudioEngine):
   <https://github.com/RustAudio/cpal> src/host/coreaudio/ios/`mod.rs`

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Sources checked:
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/runtime/BUILD.
  gn (lines 31-38:
   group("libdart") selects libdart_aotruntime for profile/release;
   else/debug links libdart_jit + //flutter/lib/snapshot -- confirmed verbatim);
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/runtime/dart_vm.
  cc (lines 177-179:
   DartVM:
  :
  IsRunningPrecompiledCode() returns Dart_IsPrecompiledRuntime() -- confirmed verbatim);
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/shell/platform/darwin/ios/framework/Source/FlutterDartProject.
  mm (line 103 guards App.
  framework executablePath on IsRunningPrecompiledCode() true;
   line 150 loads kernel snapshot only when !
  IsRunningPrecompiledCode(),
   i.e. debug -- confirmed verbatim);
   /tmp/agent/flutter-audit-20260612/engine/src/flutter/sky/packages/sky_engine/lib/`_embedder.yaml` (line 10:
   dart:
  ffi mapped to third_party/dart/sdk/lib/ffi/ffi.
  dart -- confirmed);
   /tmp/agent/flutter-audit-20260612/packages/flutter_tools/templates/plugin_ffi/ (ios.
  tmpl/projectName.
  podspec.
  tmpl:
   s.
  source_files='Classes/**/*' with comment confirming native sources linked into app builds;
   `README.md`.
  tmpl:
   ffigen binding generation + helper-isolate guidance for long-running calls -- confirmed);
   <https://docs.flutter.dev/platform-integration/ios/c-interop> (static link resolves via DynamicLibrary.
  executable()/process() with no dlopen;
   dynamic libs via DynamicLibrary.
  process()/open() distributed as .
  framework -- confirmed via two independent fetches incl.
   a neutral non-leading prompt)
