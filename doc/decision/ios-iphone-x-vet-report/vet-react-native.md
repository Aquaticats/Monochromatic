# iOS source-audit: React Native

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
   Managed JS engine running AHEAD-OF-TIME bytecode in a pure interpreter,
   no JIT.
   Default engine is Hermes:
   JS is compiled to Hermes Bytecode (.
  hbc) at build time by hermesc on the Mac,
   shipped in the bundle,
   and the on-device Hermes VM deserializes and interprets that bytecode (Hermes `Design.md`:
   "At runtime,
   the VM will deserialize the bytecode from the file and interpret it").
   Hermes ships as a vendored prebuilt binary framework (hermes-engine.
  podspec:
   spec.
  ios.
  vendored_frameworks = "destroot/Library/Frameworks/ios/hermesvm.
  framework";
   subspec Pre-built vendors hermesvm.
  xcframework).
   The alternative engine,
   JavaScriptCore (ReactCommon/jsc/JSCRuntime.
  cpp),
   runs in-process on iOS with its JIT disabled (Apple forbids non-Apple in-process code from writing executable memory;
   MAP_JIT needs the dynamic-codesigning entitlement only the out-of-process WKWebView holds),
   so it degrades to JSC's bytecode interpreter.
   Either way,
   app JS executes without JIT/W^X.
   Native UI is real UIKit views driven over JSI;
   native logic is plain AOT-compiled native code.
- Minimum iOS deployment:
   iOS 15.1 (hermes-engine.
  podspec:
   spec.
  platforms { :
  ios => "15.1" });
   RN core min iOS deployment is set per the React podspecs' min_supported_versions and tracks the same 15.
  x floor
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   React Native clears all three iOS platform walls by design:
   its default engine Hermes is a no-JIT ahead-of-time-BYTECODE interpreter (so app code needs no executable memory,
   wall-2 PASS,
   source:
   hermes-engine.
  podspec vendored hermesvm.
  framework + hermesc build-time .
  hbc + Hermes `Design.md`),
   and its TurboModule/JSI bridge is a pure C++ jsi:
  :
  HostObject layer (TurboModule.
  h:
  46,
   ObjCTurboModule in RCTTurboModule.
  h:
  52,
   React-jsi dependency) that links and calls a Go/Rust staticlib directly,
   which means BOTH kopia (linked c-archive/staticlib)
   and the music-player's symphonia+cpal Rust core (cpal already has a CoreAudio backend) are reusable behind RN via
   FFI without an AVAudioEngine rewrite.
   The hard part is not RN;
   it is the shared iOS reality of no long-running background,
   forcing the multi-hour kopia snapshot onto background URLSession + BGProcessingTask.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

React Native does not need JIT or executable memory on iOS.
 The default engine Hermes is a pure bytecode INTERPRETER over AHEAD-OF-TIME compiled bytecode:
 hermesc compiles JS to .
hbc at build time (hermes.
dev docs:
 'compile JavaScript to Hermes Bytecode during build time'),
 and the device VM deserializes-and-interprets it (Hermes `Design.md`).
 This is the iOS twin of Flutter Dart-AOT and is exactly why RN defaults to Hermes (reactnative.
dev/docs/hermes:
 'Hermes is used by default by React Native').
 The fallback engine JSC also runs without JIT in-process on iOS:
 only WKWebView (out-of-process) carries the dynamic-codesigning entitlement that mmap checks before granting MAP_JIT,
 so in-app JSC drops to its interpreter.
 No W^X/execmem entitlement is required for app-process code.
 Wall-2 PASS.

Source:
 Hermes `Design.md` (github.
com/facebook/hermes doc/`Design.md`,
 'deserialize the bytecode ... and interpret it');
 hermes-engine.
podspec vendored hermesvm.
framework + hermesc bytecode build
(/tmp/agent/react-native-audit-20260612/packages/react-native/sdks/hermes-engine/hermes-engine.
podspec lines 50-80,127-165);
 reactnative.
dev/docs/hermes (Hermes default,
 .
hbc at build time);
 JSC in-process JIT-disabled per Apple MAP_JIT/dynamic-codesigning gating (saagarjha.
com/blog/2020/02/23 jailed-JIT-on-iOS;
 HN 40726948)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 A Go gomobile c-archive .
xcframework or a Rust staticlib (crate-type=staticlib,
 cbindgen header) is linked into the iOS app and called via the TurboModule/JSI native-module bridge.
 JSI is a pure C++ interface (React-jsi pod):
 a TurboModule is a C++ jsi:
:
HostObject (ReactCommon/react/nativemodule/core/ReactCommon/TurboModule.
h:
 'class JSI_EXPORT TurboModule :
 public jsi:
:
HostObject'),
 and on iOS the concrete bridge is ObjCTurboModule (a C++ class subclassing TurboModule) plus the RCTTurboModule ObjC
 protocol,
 compiled as ObjC++ .
mm (React-NativeModulesApple.
podspec source_files 'ReactCommon/**/*.
{mm,
cpp,
h}',
 depends on React-jsi).
 So app code paths are:
 (1) link the static .
a/.
xcframework into the Xcode target,
 (2) declare extern C functions from the Rust/Go header,
 (3) call them directly from the C++/ObjC++ TurboModule body,
 (4) marshal results to jsi:
:
Value back to JS.
 No serialization bridge.
 This is the same C++-link-then-call pattern Kotlin/cinterop and dart:
ffi use,
 exposed here as JSI/TurboModule.

Source:
 /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/
 ReactCommon/TurboModule.
h:
46 (TurboModule :
 jsi:
:
HostObject);
 ReactCommon/react/nativemodule/core/platform/ios/
 ReactCommon/RCTTurboModule.
h:
52 (ObjCTurboModule :
 public TurboModule) and :
191 (@protocol RCTTurboModule);
 ReactCommon/react/nativemodule/core/platform/ios/React-NativeModulesApple.
podspec (source_files .
mm/.
cpp,
 dependency React-jsi)

## Wall 3: background execution

Same iOS constraints every candidate inherits;
 no foreground-service equivalent.
 RN core exposes no long-running background primitive,
 so a multi-hour kopia snapshot must be restructured around iOS background URLSession
 (resumable background uploads survive app suspension) and BGProcessingTask (short,
 idle/charging windows,
 ~30s practical execution).
 These are reached through a native (TurboModule) wrapper since core RN does not bind them;
 the community already does this (react-native-background-upload bridges NSURLSession background uploads;
 react-native-background-actions).
 kopia's snapshot pipeline cannot run as an unbounded foreground task;
 it must be chunked so transfer is handed to background URLSession and resumed across BGProcessingTask wakeups,
 exactly as on the Kotlin/Android stack but with even tighter iOS budgets.

Source:
 reactnative.
dev native-module pattern (no core background API;
 bridged via TurboModule);
 community NSURLSession bridges:
 github.
com/Vydia/react-native-background-upload,
 npmjs.
com/package/react-native-background-actions;
 Apple BGProcessingTask / background URLSession constraints (72technologies.
com/blog/react-native-background-tasks-ios-android-2026)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 An in-app local HTTP/S3 endpoint that kopia targets is not in RN core but is straightforward:
 either (a) embed the server inside the linked Go/Rust static lib itself (e.g. kopia's own server,
 or a Rust hyper/axum listener) bound to 127.0.0.1 and started from the TurboModule,
 or (b) use an ObjC/Swift socket server (GCDWebServer-style) wrapped as a TurboModule.
 Because the server is plain native code linked into the app (not an exec'd binary),
 it satisfies wall 1.
 kopia would point at <http://127.0.0.1>:
<port>.
 Loopback servers are permitted in-app on iOS.

Source:
 /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/platform/ios/
 ReactCommon/RCTTurboModule.
h (ObjCTurboModule bridge for native server module);
 same JSI link mechanism as wall1 (TurboModule.
h:
46)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two routes,
 both linked-native (no exec).
 (1) Reuse the Go/Rust static lib's own HTTPS client (Go net/http,
 or Rust reqwest/hyper with rustls) to stream to pCloud,
 driven from the TurboModule,
 keeping streaming/backpressure inside native code.
 (2) For transfers that must survive backgrounding,
 hand the upload to iOS background URLSession via a thin TurboModule wrapper.
 RN's own JS fetch (whatwg-fetch polyfill over the native networking module) is fine for control-plane calls
 but is not the right tool for multi-GB streaming;
 the streaming body belongs in the linked native client or background URLSession.

Source:
 reactnative.
dev native-module/TurboModule bridging (RCTTurboModule.
h ObjCTurboModule);
 Apple background URLSession for resumable uploads (72technologies.
com/blog/react-native-background-tasks-ios-android-2026);
 RN core networking is a native module,
 not a streaming uploader

## Audio (music-player port)

Decode:
 symphonia 0.6 (+ libopus) reused as-is inside the linked Rust static lib;
 React Native core ships NO audio decoder (repo audio grep over React/ and Libraries/ finds only RCTVibration.
mm using AudioToolbox for haptics,
 not playback).

Output:
 cpal 0.18,
 which has an iOS/CoreAudio (AudioUnit/RemoteIO) backend,
 reused as-is for low-latency output;
 a small AVAudioSession TurboModule activates the playback audio category (unavoidable iOS glue on any framework).
 True-peak normalization,
 peak cache,
 queue/pagination,
 and session persistence stay in the Rust core.

Rust core reuse:
 REUSED via FFI,
 not rewritten on AVAudioEngine.
 The entire symphonia+cpal core compiles to a Rust staticlib and is called from a C++/ObjC++ TurboModule (jsi:
:
HostObject) exactly as in wall 1;
 cpal already routes to CoreAudio on Apple targets,
 so the audio engine itself needs no rewrite.
 React Native contributes only the React/UIKit UI shell,
 JS-driven via JSI,
 plus the thin AVAudioSession activation module.
 This is the strongest reuse story among the walls:
 the music-player Rust core ports essentially unchanged behind a TurboModule.

Source:
 Audio absence in core:
 rg over /tmp/agent/react-native-audit-20260612/packages/react-native/React
 and Libraries (only Libraries/Vibration/RCTVibration.
mm).
 FFI path:
 TurboModule.
h:
46 (jsi:
:
HostObject) + RCTTurboModule.
h:
52 (ObjCTurboModule).
 cpal CoreAudio backend:
 cpal crate src/host/coreaudio (Apple target) per cpal docs.

## Gate probe and toolchain

Minimal app:
 `npx @react-native-community/cli init GateProbe` (New Architecture + Hermes default),
 add ONE C++ TurboModule that links a tiny Rust staticlib exporting `extern "C" int rust_add(int,int)`
 (built for aarch64-apple-ios via cargo + lipo/xcframework),
 call it from a button,
 and render the returned value.
 Build for a real iPhone (Release,
 Hermes) and launch on-device.
 Exact on-device signals confirming viability:
 (1) the app launches and `global.HermesInternal` is truthy (proves Hermes AOT-bytecode interpreter is the live engine,
 wall-2 pass);
 (2) tapping the button shows the value computed by the linked Rust staticlib
 (proves wall-1 static-lib link + JSI/TurboModule FFI works on device);
 (3) no dynamic-codesigning/JIT entitlement is present
 and no EXC_BAD_ACCESS/codesign-kill occurs (proves no executable memory is needed).
 Refutes viability if the Rust symbol fails to link/codesign,
 or if the app is killed for an execmem/JIT violation.

Toolchain:
 macOS with Xcode + iOS SDK and a provisioning profile/dev cert (device build);
 Node.
js + npm/yarn and the React Native community CLI;
 CocoaPods (Ruby) for pod install;
 Watchman (recommended) and Metro bundler;
 the Rust toolchain with the aarch64-apple-ios target (`rustup target add aarch64-apple-ios`) plus cargo + cbindgen
 and lipo/xcodebuild -create-xcframework to package the staticlib (or Go + gomobile for a c-archive .
xcframework).
 hermesc runs on the Mac at build time (no extra install;
 bundled via hermes-engine pod / hermes-compiler npm).

## Supporting-stack vets this framework drags in

- JSI/TurboModule FFI binding vet:
   Rust/Go staticlib -> cbindgen/gomobile header -> C++ jsi:
  :
  HostObject TurboModule,
   including threading (CallInvoker/JS thread vs native method-call invoker) and large-buffer marshaling without copies
- In-app loopback HTTP/S3 server vet:
   embed kopia/Rust server in the linked lib bound to 127.0.0.1,
   lifecycle from a TurboModule,
   App Transport Security exception for loopback
- HTTPS streaming + background-transfer vet:
   native-lib rustls/Go client vs iOS background URLSession bridge,
   resumable multi-GB uploads,
   BGProcessingTask scheduling under idle/charging
- kopia static-lib packaging vet:
   Go gomobile c-archive .
  xcframework (or Rust staticlib) for aarch64-apple-ios,
   symbol-stripping,
   bitcode/codesign,
   fat-vs-xcframework device+sim slices
- Audio TurboModule vet:
   symphonia+cpal staticlib behind a TurboModule,
   AVAudioSession category activation,
   interruption/route-change handling,
   true-peak cache on-disk persistence,
   low-latency RemoteIO callback under JS/UI load
- Background execution vet:
   BGProcessingTask + background URLSession orchestration of a chunked kopia snapshot,
   state machine for resume across wakeups
- QA vets:
   in-process UI test via XCUITest / Jest+react-test-renderer for the JS shell;
   Detox or Maestro e2e on-device;
   native fuzz (cargo-fuzz/libFuzzer) of the Rust core;
   mutation testing (cargo-mutants for Rust,
   Stryker for the JS layer);
   property tests (proptest for the Rust core,
   fast-check for JS)

## Cited sources

- Hermes ships as a vendored prebuilt binary framework,
   not source linked into the app,
   and is built from JS via hermesc at build time:
   /tmp/agent/react-native-audit-20260612/packages/react-native/sdks/hermes-engine/hermes-engine.
  podspec (spec.
  ios.
  vendored_frameworks hermesvm.
  framework line ~50;
   Pre-built subspec hermesvm.
  xcframework;
   hermesc bytecode build lines 50-80,127-165)
- Hermes is a bytecode interpreter over ahead-of-time compiled bytecode (no JIT):
   github.
  com/facebook/hermes doc/`Design.md` ('At runtime,
   the VM will deserialize the bytecode from the file and interpret it');
   reactnative.
  dev/docs/hermes ('Hermes is used by default';
   compile to Hermes Bytecode at build time)
- In-process JSC on iOS has JIT disabled;
   only out-of-process WKWebView holds the dynamic-codesigning entitlement gating MAP_JIT:
   saagarjha.
  com/blog/2020/02/23/jailed-just-in-time-compilation-on-ios/;
   Hacker News 40726948 ('JavaScriptCore doesn't use a JIT when used in your own apps');
   JSC runtime exists at /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/jsc/JSCRuntime.
  cpp
- TurboModules are C++ jsi:
  :
  HostObjects exposed over JSI (a pure C++ interface),
   giving a direct FFI path to linked native static libs:
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/
   ReactCommon/TurboModule.
  h:
  46
- On iOS the concrete native-module bridge is ObjCTurboModule (C++ subclass) + RCTTurboModule protocol,
   compiled as ObjC++ and depending on React-jsi:
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/platform/ios/
   ReactCommon/RCTTurboModule.
  h:
  52,191 and React-NativeModulesApple.
  podspec (source_files .
  mm/.
  cpp,
   dependency React-jsi)
- React Native core ships no audio decoder or playback engine (only haptics via AudioToolbox),
   so symphonia+cpal must be supplied as a native module and is reusable via FFI:
   rg over /tmp/agent/react-native-audit-20260612/packages/react-native/React and Libraries:
   only Libraries/Vibration/RCTVibration.
  mm references AudioToolbox
- Background transfer must use iOS background URLSession + BGProcessingTask;
   no foreground-service equivalent;
   community bridges NSURLSession via native modules:
   github.
  com/Vydia/react-native-background-upload;
   npmjs.
  com/package/react-native-background-actions;
   72technologies.
  com/blog/react-native-background-tasks-ios-android-2026
- Minimum iOS deployment for the Hermes engine pod is iOS 15.1:
   /tmp/agent/react-native-audit-20260612/packages/react-native/sdks/hermes-engine/hermes-engine.
  podspec (spec.
  platforms { :
  ios => '15.1' })

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   All load-bearing claims are supported;
   no fabricated or contradicted claims found.
   Only minor source-attribution imprecisions (do NOT change the verdict):

1) SOURCE MISLABEL (quote correct,
    attribution off):
    The audit's wall-2 attributes "compile JavaScript to Hermes Bytecode during build time" to "hermes.
   dev docs".
    The verbatim sentence
    ("This will compile JavaScript to Hermes Bytecode during build time
which will improve your app's startup speed on device") actually lives on reactnative.
   dev/docs/hermes,
    which the audit also separately cites.
    The quote is accurate;
    only the doc name is loose.

2) SCOPE OF `Design.md` (not a contradiction):
    The audit's iosRuntimeModel implies Hermes `Design.md` covers the full build-then-interpret pipeline.
    `Design.md` confirms verbatim "At runtime,
    the VM will deserialize the bytecode from the file and interpret it" but does NOT itself name hermesc
    or describe build-time compilation;
    that half ("hermesc compiles JS to .
   hbc at build time") is supported by reactnative.
   dev/docs/hermes and the hermes-engine.
   podspec (HERMES_CLI_PATH -> hermesc,
    lines 74/134),
    not by `Design.md`.
    Both halves are sourced correctly elsewhere,
    so the composite claim stands.

3) URL SLUG (post exists):
    The saagarjha citation slug "jailed-JIT-on-iOS" is a paraphrase;
    real post is saagarjha.
   com/blog/2020/02/23/jailed-just-in-time-compilation-on-ios/.
    Content confirms the load-bearing claim:
    "Just-in-time compilation on iOS normally requires applications to possess the dynamic-codesigning entitlement,
    a privilege that Apple uniquely awards to system processes"
    and third-party apps "cannot use mmap's MAP_JIT without this entitlement",
    so in-app JSC runs JIT-disabled.

4) SHORTHAND (substantively correct):
    Audit says "only WKWebView (out-of-process) carries the dynamic-codesigning entitlement.
   " The entitlement is held by Apple system processes (Safari/WebContent);
    WKWebView's out-of-process WebContent is the mechanism third-party apps get JIT JS,
    so "WKWebView" is acceptable shorthand,
    not a fabrication.

NOTE:
 The RN repo itself contains NO JIT-entitlement handling code
 (grep for useJIT/MAP_JIT/allow-jit/dynamic-codesigning returned zero hits in package/react-native).
 That is expected (it is OS-level behavior),
 and the audit correctly sources that sub-claim to external Apple/saagarjha references rather than the repo.

The HN corroborating citation (id 40726948) could not be fetched (HTTP 429 rate-limit),
 but it is secondary;
 the primary saagarjha source and Apple entitlement docs already confirm the JSC-JIT-gating claim.
- Sources checked:
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/
   ReactCommon/TurboModule.
  h (line 46:
   class JSI_EXPORT TurboModule :
   public jsi:
  :
  HostObject - CONFIRMED verbatim);
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/platform/ios/
   ReactCommon/RCTTurboModule.
  h (line 52:
   class JSI_EXPORT ObjCTurboModule :
   public TurboModule;
   line 191:
   @protocol RCTTurboModule <RCTModuleProvider> - both CONFIRMED verbatim);
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/react/nativemodule/core/platform/ios/
   React-NativeModulesApple.
  podspec (line 37:
   source_files ReactCommon/**/*.
  {mm,
  cpp,
  h};
   line 44:
   s.
  dependency React-jsi - CONFIRMED);
   /tmp/agent/react-native-audit-20260612/packages/react-native/sdks/hermes-engine/hermes-engine.
  podspec (line 51:
   ios.
  vendored_frameworks hermesvm.
  framework;
   line 62:
   subspec Pre-built vendors hermesvm.
  xcframework;
   lines 74/134:
   hermesc path / HERMES_CLI_PATH - CONFIRMED);
   /tmp/agent/react-native-audit-20260612/packages/react-native/ReactCommon/jsc/JSCRuntime.
  cpp (exists,
   includes JavaScriptCore/JavaScript.
  h,
   class JSCRuntime :
   public jsi:
  :
  Runtime - CONFIRMED engine path);
   rg over package/react-native for useJIT|MAP_JIT|dynamic-codesigning|allow-jit - ZERO hits
   (JIT-gating is correctly sourced externally,
   not from repo);
   <https://reactnative.dev/docs/hermes> (CONFIRMED verbatim:
   'Hermes is used by default by React Native and no additional configuration is required to enable it.
  ' and 'This will compile JavaScript to Hermes Bytecode during build time');
   <https://github.com/facebook/hermes/blob/main/doc/Design.md> (CONFIRMED verbatim:
   'At runtime,
   the VM will deserialize the bytecode from the file and interpret it.
  ' Does NOT itself name hermesc/build-time compile);
   <https://saagarjha.com/blog/2020/02/23/jailed-just-in-time-compilation-on-ios/> (real URL;
   CONFIRMED:
   dynamic-codesigning entitlement uniquely awarded to system processes;
   third-party apps cannot use mmap MAP_JIT without it -> in-app JSC JIT-disabled);
   WebSearch corroboration (HN 18429757,
   Apple allow-jit entitlement docs):
   dynamic-codesigning gates MAP_JIT,
   only Safari/Apple apps permitted -> JavaScriptCore.
  framework cannot JIT locally in third-party apps;
   <https://news.ycombinator.com/item?id=40726948> (secondary corroborating citation - NOT fetched,
   HTTP 429 rate-limit;
   primary sources already confirm the claim)
