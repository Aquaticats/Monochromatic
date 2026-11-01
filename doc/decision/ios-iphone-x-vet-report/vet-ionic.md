# iOS source-audit: Ionic Framework

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
   WKWebView-JIT.
   Ionic is pure web technology:
   Stencil-compiled web components (HTML/CSS/JS) with zero native code of its own (find under
   /tmp/agent/ionic-audit-20260612 returns 0 .
  swift/.
  m/Podfile/.
  podspec files).
   On an iOS device the entire Ionic UI is an out-of-process WKWebView;
   app JS executes in WebKit's JavaScriptCore engine,
   which legitimately JITs because the WebContent process holds the dynamic-codesigning entitlement.
   Ionic detects iOS purely by user-agent string (core/src/utils/`platform.ts`:
  66 isIOS = testUserAgent(/iPhone|iPod/i) || isIpad) and reaches native capability only through window.
  Capacitor (core/src/utils/native/`capacitor.ts` getCapacitor).
   Crucially the JIT lives in the sandboxed WKWebView,
   not the app process,
   so unlike NativeScript (which embedded a JS engine IN the app process
   and hit DENY_EXECMEM) Ionic does not need an app-process JIT at all.
- Minimum iOS deployment:
   Capacitor 7 (current) requires iOS 14.0+ (project IPHONEOS_DEPLOYMENT_TARGET 14.0);
   iPhone X tops out at iOS 16,
   so it is within range.
   Ionic itself imposes no harder floor (web-component support tracks the installed WKWebView).
   needs-device confirmation of the exact deployment target in the generated Xcode project.
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Ionic carries no app-process runtime at all:
   it is Stencil web components (0 native source files in the repo) whose JS runs in the out-of-process WKWebView (JIT
   lives in WebKit,
   which alone holds the dynamic-codesigning entitlement per ExecutableAllocator.
  cpp),
   and whose only native reach is window.
  Capacitor.
   So Ionic clears wall 2 by architecture,
   and every load-bearing capability (kopia static-lib FFI,
   in-app HTTP server,
   streaming/background URLSession,
   symphonia+cpal audio reuse) is delivered by Capacitor Swift/ObjC plugins,
   NOT by Ionic.
   The decision about Ionic is really a decision about the Capacitor substrate plus a UI layer that is just HTML/CSS/JS
   in a webview;
   the Rust music-player core is reusable via FFI but only from a Capacitor plugin in the app process,
   never inside the webview.

## Wall 2: JIT / executable memory

Verdict:
 wkwebview-jit

Ionic app code (JS) runs inside WKWebView,
 the one out-of-process surface Apple grants the dynamic-codesigning / com.
apple.
developer.
cs.
allow-jit entitlement.
 WebKit's ExecutableAllocator gates JIT on exactly these entitlements:
 jitEnabled = jitEnabled && (processHasEntitlement("dynamic-codesigning") || processHasEntitlement("com.
apple.
developer.
cs.
allow-jit"));
 arbitrary app processes cannot allocate W^X/JIT memory,
 only the WebContent process can.
 Ionic never compiles or interprets app code in the app process,
 so wall 2 is satisfied by architecture:
 the JS engine that does need execmem is the very WKWebView Apple sanctions.
 No managed/AOT step is required and no interpreter fallback is needed.

Source:
 WebKit Source/JavaScriptCore/jit/ExecutableAllocator.
cpp (processHasEntitlement('dynamic-codesigning')/'com.
apple.
developer.
cs.
allow-jit' gate);
 /tmp/agent/ionic-audit-20260612/core/src/utils/`platform.ts`:
66 and core/src/utils/native/`capacitor.ts` confirm Ionic carries no app-process runtime

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Not in Ionic itself (Ionic has zero native code;
 only window.
Capacitor touchpoint at core/src/utils/native/`capacitor.ts`).
 Linking kopia (Go gomobile c-archive .
xcframework or Rust staticlib) is a Capacitor-shell job:
 you author a Capacitor iOS plugin (Swift/ObjC class extending CAPPlugin + CAPBridgedPlugin) that links the static lib
 and calls its C ABI;
 native plugin code runs IN the app process while JS runs in WKWebView.
 JS reaches it via the Promise bridge:
 a CAPPluginMethod registered in pluginMethods,
 invoked through a CAPPluginCall,
 returning via call.
resolve()/call.
reject().
 So the FFI path is JS -> Capacitor bridge -> Swift/ObjC plugin -> C/Go/Rust staticlib.
 This is the SAME native-substrate capability Capacitor/Cordova provide;
 Ionic adds nothing and blocks nothing at this wall.

Source:
 <https://capacitorjs.com/docs/plugins/ios> (CAPPlugin/CAPBridgedPlugin,
 pluginMethods array of CAPPluginMethod,
 CAPPluginCall,
 call.
resolve();
 plugin runs in app process while JS runs in WKWebView);
 /tmp/agent/ionic-audit-20260612/core/src/utils/native/`capacitor.ts` (Ionic's only native bridge is window.
Capacitor)

## Wall 3: background execution

Inherited entirely from the Capacitor/Cordova substrate;
 Ionic has no background machinery (it is UI web components).
 A multi-hour kopia snapshot cannot run as a foreground service.
 It must be restructured around iOS background URLSession (out-of-process,
 system-driven uploads/downloads that survive app suspension) and BGProcessingTask (short,
 opportunistic,
 idle/charging) exposed through a Capacitor plugin or a Capacitor-community background plugin.
 From the web/JS layer there is no way to hold a long-running background thread;
 the orchestration and the URLSession delegate live in native Swift in the Capacitor plugin,
 with JS only kicking off and polling status across the bridge.

Source:
 /tmp/agent/ionic-audit-20260612 has no background-task code (UI-only repo,
 0 native files);
 background is a substrate concern per <https://capacitorjs.com/docs/plugins/ios> bridging model
 and Apple background URLSession/BGTaskScheduler APIs

## In-app HTTP server (kopia S3 target)

Feasible:
 partial

Mechanism:
 Not from Ionic/WKWebView JS (no socket-listen API in the web layer).
 Must be a native listener started by a Capacitor plugin in the app process:
 Swift Network.
framework / GCDWebServer,
 or the kopia repository server compiled into the linked staticlib,
 bound to loopback for kopia to target;
 JS configures it over the bridge.

Source:
 /tmp/agent/ionic-audit-20260612/core/src/utils/native/`capacitor.ts` (only native reach is window.
Capacitor);
 <https://capacitorjs.com/docs/plugins/ios> (Swift plugin runs native code in the app process)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two routes,
 both substrate.
 From the web layer,
 fetch()/XMLHttpRequest in WKWebView speak HTTPS but are not ideal for large streaming/background.
 The robust route is native:
 a Capacitor plugin using URLSession (uploadTask/downloadTask with streaming bodies + background sessions) to stream to
 pCloud,
 controlled from JS via the bridge.
 Ionic adds no HTTP client of its own.

Source:
 <https://capacitorjs.com/docs/plugins/ios> (CAPPluginCall bridge to native URLSession);
 /tmp/agent/ionic-audit-20260612 has no HTTP client code

## Audio (music-player port)

Decode:
 At the Ionic/web layer there is no audio engine (the only 'audio' match in core/src is tap-click haptics,
 not playback).
 Decoding would be Web Audio / HTMLAudioElement in WKWebView (no symphonia,
 no libopus,
 no true-peak normalization,
 no on-disk peak cache available at that layer).
 To keep symphonia 0.6 + libopus you must NOT decode in the webview;
 you wrap the Rust core in a Capacitor plugin and decode natively.

Output:
 Web layer outputs through WKWebView's Web Audio API (high latency,
 no CoreAudio control).
 For low-latency output you bypass the webview entirely and drive native CoreAudio/AVAudioEngine (or cpal's CoreAudio
 backend) from a Capacitor plugin in the app process.

Rust core reuse:
 reusable-via-FFI-in-a-Capacitor-plugin,
 NOT in the webview.
 The existing Rust symphonia + cpal core (cpal has a CoreAudio iOS backend) compiles to a static lib
 and is called from a Swift/ObjC Capacitor plugin over the C ABI;
 it runs in the app process,
 not the WKWebView.
 You do NOT have to rewrite decode on AVAudioEngine,
 but you DO have to move all audio out of Ionic's web layer into a native plugin (Ionic only paints UI).
 If you instead wanted audio inside the webview,
 the Rust core could not be reused and you would fall back to Web Audio.
 Net:
 Slint/native music-player audio reuse is a Capacitor-plugin task,
 fully outside what Ionic does.

Source:
 /tmp/agent/ionic-audit-20260612/core/src (no audio playback engine;
 tap-click/`index.ts` is haptics);
 <https://capacitorjs.com/docs/plugins/ios> (native Swift plugin links C/Rust static lib,
 runs in app process);
 cpal CoreAudio (coreaudio-rs) iOS backend

## Gate probe and toolchain

Same as gateProbe field:
 build the Ionic+Capacitor blank starter,
 npx cap add ios,
 sign and install on the iPhone X,
 confirm the WKWebView UI renders and ion-button/ion-router-outlet are interactive with no codesign/execmem crash;
 then add one trivial Capacitor plugin linking a tiny Rust/Go staticlib
 and read its return value back in JS to validate the wall-1 FFI bridge.

Toolchain:
 macOS with Xcode (iOS SDK,
 codesign,
 devicectl/Instruments);
 Node.
js + npm;
 Ionic CLI (@ionic/cli) and Capacitor CLI (npx cap);
 CocoaPods for the iOS native project.
 For the FFI probe:
 Rust with the aarch64-apple-ios target (or Go + gomobile for a Go c-archive) to produce the static lib/.
xcframework linked by the Capacitor plugin.

## Supporting-stack vets this framework drags in

- Capacitor iOS substrate vet:
   CAPPlugin/CAPBridgedPlugin authoring,
   linking a kopia Go gomobile c-archive .
  xcframework (or Rust staticlib) and calling its C ABI from Swift,
   app-process vs WKWebView process model
- In-app HTTP/S3 endpoint vet:
   native listener (Network.
  framework/GCDWebServer or kopia repository-server in the staticlib) on loopback that kopia targets,
   started/stopped via the bridge
- HTTPS streaming + background-transfer vet:
   URLSession upload/download tasks with streaming bodies and background sessions to pCloud;
   BGProcessingTask scheduling;
   restructuring a multi-hour snapshot into resumable URLSession chunks
- Native audio plugin vet:
   wrapping the Rust symphonia 0.6 + libopus + cpal core as a Capacitor plugin,
   cpal CoreAudio iOS backend,
   true-peak normalization and on-disk peak cache running in the app process,
   low-latency output bypassing WKWebView Web Audio
- WKWebView <-> native bridge throughput/latency vet for the music-player UI (queue/two-axis pagination/session
persistence) talking to the native audio core
- QA vets:
   in-WKWebView component UI tests (Stencil/Playwright or Cypress against the webview),
   Appium/XCUITest e2e on device,
   JS-side fuzz/property tests for the bridge message contract,
   and native-plugin unit tests for the Swift/Rust FFI boundary

## Cited sources

- Ionic is web components with zero native iOS source;
   the only native bridge is window.
  Capacitor:
   /tmp/agent/ionic-audit-20260612/core/src/utils/native/`capacitor.ts` (getCapacitor returns window.
  Capacitor);
   find for .
  swift/.
  m/Podfile/.
  podspec returns 0
- Ionic detects iOS purely by user-agent and treats hybrid native as Cordova or Capacitor:
   /tmp/agent/ionic-audit-20260612/core/src/utils/`platform.ts`:
  66 (isIOS via testUserAgent),
   :
  96-:
  101 (isHybrid = isCordova || isCapacitorNative)
- App JS runs in WKWebView,
   which is the only iOS surface granted JIT;
   arbitrary app processes cannot allocate W^X memory:
   WebKit Source/JavaScriptCore/jit/ExecutableAllocator.
  cpp gate:
   processHasEntitlement('dynamic-codesigning') || processHasEntitlement('com.
  apple.
  developer.
  cs.
  allow-jit')
- Capacitor iOS plugins are Swift/ObjC (CAPPlugin/CAPBridgedPlugin),
   run native code in the app process,
   and are invoked from JS via CAPPluginCall with Promise resolve/reject;
   native code can link a C/Go/Rust static lib:
   <https://capacitorjs.com/docs/plugins/ios>
- Ionic has no audio playback engine;
   only a tap-click/haptics module references audio-like APIs:
   /tmp/agent/ionic-audit-20260612/core/src grep AudioContext/HTMLAudioElement -> only tap-click/`index.ts` (haptics)
- @ionic/core version and Stencil-based architecture:
   /tmp/agent/ionic-audit-20260612/core/`package.json` (name @ionic/core,
   version 8.8.10,
   dep @stencil/core 4.43.5);
   `README.md`:
  15 'Ionic is based on Web Components'

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   No substantive errors found;
   both walls' load-bearing claims are supported by their cited sources.
   One minor precision note (not a contradiction,
   verdict stands):
   the audit presents the JIT gate as the bare expression "jitEnabled = jitEnabled &&
   (processHasEntitlement('dynamic-codesigning') || processHasEntitlement('com.
  apple.
  developer.
  cs.
  allow-jit'))".
   The real WebKit source (Source/JavaScriptCore/jit/ExecutableAllocator.
  cpp,
   isJITEnabled(),
   line 149) is identical in substance but (a) uses WTF string-literal suffix "_s" on each entitlement string,
   and (b) wraps that line in "#if HAVE(IOS_JIT_RESTRICTIONS)" with the base term "bool jitEnabled = !
  g_jscConfig.
  jitDisabled;
  " (line 147).
   The audit's AND-with-base / OR-of-two-entitlements characterization is correct,
   and the macOS-internal-SDK branch uses a different single entitlement ("com.
  apple.
  security.
  cs.
  allow-jit",
   line 151).
   The "only the WebContent process holds the entitlement / arbitrary app processes cannot JIT" statement is the
   audit's architectural framing rather than a verbatim quote from ExecutableAllocator.
  cpp;
   the code only proves JIT requires one of those entitlements in the running process,
   which is consistent with and supports the framing.
- Sources checked:
   /tmp/agent/ionic-audit-20260612/core/src/utils/`platform.ts`:
  66 (verified:
   isIOS = testUserAgent(/iPhone|iPod/i) || isIpad,
   UA-string iOS detection,
   exact match);
   /tmp/agent/ionic-audit-20260612/core/src/utils/native/`capacitor.ts` (verified:
   getCapacitor reads window.
  Capacitor,
   Ionic's only native touchpoint,
   no app-process runtime);
   find under /tmp/agent/ionic-audit-20260612 for .
  swift/.
  m/Podfile/.
  podspec (verified:
   0 results,
   confirms zero native code);
   WebKit Source/JavaScriptCore/jit/ExecutableAllocator.
  cpp raw source (verified lines 145-153:
   isJITEnabled() gate quote matches,
   line 149 OR of dynamic-codesigning/com.
  apple.
  developer.
  cs.
  allow-jit under HAVE(IOS_JIT_RESTRICTIONS));
   <https://capacitorjs.com/docs/plugins/ios> (verified all 6 wall-1 identifiers:
   CAPPlugin,
   CAPBridgedPlugin,
   pluginMethods:
  [CAPPluginMethod],
   CAPPluginCall,
   call.
  resolve(),
   EchoPlugin example)
